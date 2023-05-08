use crate::msg::AskAsset;
use crate::msg::{DexAction, OfferAsset, SwapRouter};
use crate::state::SWAP_FEE;
use crate::{error::DexError, DEX};
use abstract_core::objects::{DexAssetPairing, PoolReference};
use abstract_sdk::core::objects::AnsAsset;
use abstract_sdk::core::objects::AssetEntry;
use abstract_sdk::cw_helpers::fees::Chargeable;
use abstract_sdk::features::AbstractNameService;
use abstract_sdk::Execution;
use cosmwasm_std::{to_binary, Addr, Coin, CosmosMsg, Decimal, Deps, StdError, StdResult, WasmMsg};
use cw20::Cw20ExecuteMsg;
use cw_asset::{Asset, AssetInfo};

pub const PROVIDE_LIQUIDITY: u64 = 7542;
pub const PROVIDE_LIQUIDITY_SYM: u64 = 7543;
pub const WITHDRAW_LIQUIDITY: u64 = 7546;
pub const SWAP: u64 = 7544;
pub const CUSTOM_SWAP: u64 = 7545;

impl<T> LocalDex for T where T: AbstractNameService + Execution {}

pub(crate) type ReplyId = u64;

pub trait LocalDex: AbstractNameService + Execution {
    /// resolve the provided dex action on a local dex
    fn resolve_dex_action(
        &self,
        deps: Deps,
        action: DexAction,
        exchange: &dyn DEX,
    ) -> Result<(Vec<CosmosMsg>, ReplyId), DexError> {
        Ok(match action {
            DexAction::ProvideLiquidity { assets, max_spread } => {
                if assets.len() < 2 {
                    return Err(DexError::TooFewAssets {});
                }
                (
                    self.resolve_provide_liquidity(deps, assets, exchange, max_spread)?,
                    PROVIDE_LIQUIDITY,
                )
            }
            DexAction::ProvideLiquiditySymmetric {
                offer_asset,
                paired_assets,
            } => {
                if paired_assets.is_empty() {
                    return Err(DexError::TooFewAssets {});
                }
                (
                    self.resolve_provide_liquidity_symmetric(
                        deps,
                        offer_asset,
                        paired_assets,
                        exchange,
                    )?,
                    PROVIDE_LIQUIDITY_SYM,
                )
            }
            DexAction::WithdrawLiquidity { lp_token, amount } => (
                self.resolve_withdraw_liquidity(deps, AnsAsset::new(lp_token, amount), exchange)?,
                WITHDRAW_LIQUIDITY,
            ),
            DexAction::Swap {
                offer_asset,
                ask_asset,
                max_spread,
                belief_price,
            } => (
                self.resolve_swap(
                    deps,
                    offer_asset,
                    ask_asset,
                    exchange,
                    max_spread,
                    belief_price,
                )?,
                SWAP,
            ),
            DexAction::CustomSwap {
                offer_assets,
                ask_assets,
                max_spread,
                router,
            } => (
                self.resolve_custom_swap(
                    deps,
                    offer_assets,
                    ask_assets,
                    exchange,
                    max_spread,
                    router,
                )?,
                CUSTOM_SWAP,
            ),
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn resolve_swap(
        &self,
        deps: Deps,
        offer_asset: OfferAsset,
        mut ask_asset: AssetEntry,
        exchange: &dyn DEX,
        max_spread: Option<Decimal>,
        belief_price: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let AnsAsset {
            name: mut offer_asset,
            amount: offer_amount,
        } = offer_asset;
        offer_asset.format();
        ask_asset.format();

        let ans = self.name_service(deps);
        let offer_asset_info = ans.query(&offer_asset)?;
        let ask_asset_info = ans.query(&ask_asset)?;

        let pair_address =
            exchange.pair_address(deps, ans.host(), (offer_asset.clone(), ask_asset))?;
        let mut offer_asset: Asset = Asset::new(offer_asset_info, offer_amount);
        // account for fee
        let fee = SWAP_FEE.load(deps.storage)?;
        let fee_msg = offer_asset.charge_usage_fee(fee)?;
        let mut swap_msgs = exchange.swap(
            deps,
            pair_address,
            offer_asset,
            ask_asset_info,
            belief_price,
            max_spread,
        )?;
        // insert fee msg
        if let Some(f) = fee_msg {
            swap_msgs.push(f)
        }

        Ok(swap_msgs)
    }

    #[allow(clippy::too_many_arguments)]
    fn resolve_custom_swap(
        &self,
        _deps: Deps,
        _offer_assets: Vec<OfferAsset>,
        _ask_assets: Vec<AskAsset>,
        _exchange: &dyn DEX,
        _max_spread: Option<Decimal>,
        _router: Option<SwapRouter>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        todo!()

        // let ans_host = api.ans(deps);
        //
        // // Resolve the asset information
        // let mut offer_asset_infos: Vec<AssetInfo> =
        //     exchange.resolve_assets(deps, &api, offer_assets.into_iter().unzip().0)?;
        // let mut ask_asset_infos: Vec<AssetInfo> =
        //     exchange.resolve_assets(deps, &api, ask_assets.into_iter().unzip().0)?;
        //
        // let offer_assets: Vec<Asset> = offer_assets
        //     .into_iter()
        //     .zip(offer_asset_infos)
        //     .map(|(asset, info)| Asset::new(info, asset.1))
        //     .collect();
        // let ask_assets: Vec<Asset> = ask_assets
        //     .into_iter()
        //     .zip(ask_asset_infos)
        //     .map(|(asset, info)| Asset::new(info, asset.1))
        //     .collect();
        //
        // exchange.custom_swap(deps, offer_assets, ask_assets, max_spread)
    }

    fn resolve_provide_liquidity(
        &self,
        deps: Deps,
        offer_assets: Vec<OfferAsset>,
        exchange: &dyn DEX,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let ans = self.name_service(deps);
        let assets = ans.query(&offer_assets)?;

        let mut pair_assets = offer_assets
            .into_iter()
            .map(|a| a.name)
            .take(2)
            .collect::<Vec<AssetEntry>>();

        let pair_address = exchange.pair_address(
            deps,
            ans.host(),
            (pair_assets.swap_remove(0), pair_assets.swap_remove(0)),
        )?;
        exchange.provide_liquidity(deps, pair_address, assets, max_spread)
    }

    fn resolve_provide_liquidity_symmetric(
        &self,
        deps: Deps,
        offer_asset: OfferAsset,
        mut paired_assets: Vec<AssetEntry>,
        exchange: &dyn DEX,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let ans = self.name_service(deps);
        let paired_asset_infos = ans.query(&paired_assets)?;
        let pair_address = exchange.pair_address(
            deps,
            ans.host(),
            (paired_assets.swap_remove(0), paired_assets.swap_remove(1)),
        )?;
        let offer_asset = ans.query(&offer_asset)?;
        exchange.provide_liquidity_symmetric(deps, pair_address, offer_asset, paired_asset_infos)
    }

    /// @todo
    fn resolve_withdraw_liquidity(
        &self,
        deps: Deps,
        lp_token: OfferAsset,
        exchange: &dyn DEX,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let ans = self.name_service(deps);

        let lp_asset = ans.query(&lp_token)?;

        let lp_pairing: DexAssetPairing = lp_token.name.try_into()?;

        let mut pool_ids = ans.query(&lp_pairing)?;
        // TODO: when resolving if there are more than one, get the metadata and choose the one matching the assets
        if pool_ids.len() != 1 {
            return Err(StdError::generic_err(format!(
                "There are {} pairings for the given LP token",
                pool_ids.len()
            ))
            .into());
        }

        let PoolReference { pool_address, .. } = pool_ids.pop().unwrap();
        exchange.withdraw_liquidity(deps, pool_address, lp_asset)
    }
}
pub(crate) fn cw_approve_msgs(assets: &[Asset], spender: &Addr) -> StdResult<Vec<CosmosMsg>> {
    let mut msgs = vec![];
    for asset in assets {
        if let AssetInfo::Cw20(addr) = &asset.info {
            let msg = Cw20ExecuteMsg::IncreaseAllowance {
                spender: spender.to_string(),
                amount: asset.amount,
                expires: None,
            };
            msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: addr.to_string(),
                msg: to_binary(&msg)?,
                funds: vec![],
            }))
        }
    }
    Ok(msgs)
}

pub(crate) fn coins_in_assets(assets: &[Asset]) -> Vec<Coin> {
    let mut coins = vec![];
    for asset in assets {
        if let AssetInfo::Native(denom) = &asset.info {
            coins.push(Coin::new(asset.amount.u128(), denom.clone()));
        }
    }
    coins
}
