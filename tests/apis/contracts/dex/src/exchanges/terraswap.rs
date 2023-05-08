use crate::{
    commands::{coins_in_assets, cw_approve_msgs},
    dex_trait::{Fee, FeeOnInput, Identify, Return, Spread},
    error::DexError,
    DEX,
};
use abstract_core::objects::PoolAddress;
use abstract_sdk::cw_helpers::cosmwasm_std::wasm_smart_query;
use cosmwasm_std::{to_binary, wasm_execute, CosmosMsg, Decimal, Deps};
use cw20::Cw20ExecuteMsg;
use cw_asset::{Asset, AssetInfo, AssetInfoBase};
use terraswap::pair::{PoolResponse, SimulationResponse};

pub const TERRASWAP: &str = "terraswap";

pub struct Terraswap {}

impl Identify for Terraswap {
    fn over_ibc(&self) -> bool {
        false
    }
    fn name(&self) -> &'static str {
        TERRASWAP
    }
}

impl DEX for Terraswap {
    fn swap(
        &self,
        _deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        _ask_asset: AssetInfo,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pair_address = pool_id.expect_contract()?;

        let proxy_msg = if let AssetInfoBase::Cw20(token_addr) = &offer_asset.info {
            let hook_msg = terraswap::pair::Cw20HookMsg::Swap {
                belief_price,
                max_spread,
                to: None,
            };
            // Call swap on pair through cw20 Send
            let send_msg = Cw20ExecuteMsg::Send {
                contract: pair_address.to_string(),
                amount: offer_asset.amount,
                msg: to_binary(&hook_msg)?,
            };
            // call send on cw20
            wasm_execute(token_addr, &send_msg, vec![])?
        } else {
            let swap_msg = terraswap::pair::ExecuteMsg::Swap {
                offer_asset: cw_asset_to_terraswap(&offer_asset)?,
                max_spread,
                belief_price,
                to: None,
            };
            wasm_execute(pair_address, &swap_msg, coins_in_assets(&[offer_asset]))?
        };

        Ok(vec![proxy_msg.into()])
    }

    fn provide_liquidity(
        &self,
        _deps: Deps,
        pool_id: PoolAddress,
        offer_assets: Vec<Asset>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pair_address = pool_id.expect_contract()?;

        if offer_assets.len() > 2 {
            return Err(DexError::TooManyAssets(2));
        }

        let terraswap_assets = offer_assets
            .iter()
            .map(cw_asset_to_terraswap)
            .collect::<Result<Vec<_>, _>>()?;
        // execute msg
        let msg = terraswap::pair::ExecuteMsg::ProvideLiquidity {
            assets: [terraswap_assets[0].clone(), terraswap_assets[1].clone()],
            slippage_tolerance: max_spread,
            receiver: None,
        };
        // approval msgs for cw20 tokens (if present)
        let mut msgs = cw_approve_msgs(&offer_assets, &pair_address)?;
        let coins = coins_in_assets(&offer_assets);
        // actual call to pair
        let liquidity_msg = wasm_execute(pair_address, &msg, coins)?.into();
        msgs.push(liquidity_msg);

        Ok(msgs)
    }

    fn provide_liquidity_symmetric(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        paired_assets: Vec<AssetInfo>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pair_address = pool_id.expect_contract()?;

        if paired_assets.len() > 1 {
            return Err(DexError::TooManyAssets(2));
        }
        // Get pair info
        let pair_config: PoolResponse = deps.querier.query(&wasm_smart_query(
            pair_address.to_string(),
            &terraswap::pair::QueryMsg::Pool {},
        )?)?;

        let ts_offer_asset = cw_asset_to_terraswap(&offer_asset)?;
        let other_asset = if pair_config.assets[0].info == ts_offer_asset.info {
            let price =
                Decimal::from_ratio(pair_config.assets[1].amount, pair_config.assets[0].amount);
            let other_token_amount = price * offer_asset.amount;
            Asset {
                amount: other_token_amount,
                info: paired_assets[0].clone(),
            }
        } else if pair_config.assets[1].info == ts_offer_asset.info {
            let price =
                Decimal::from_ratio(pair_config.assets[0].amount, pair_config.assets[1].amount);
            let other_token_amount = price * offer_asset.amount;
            Asset {
                amount: other_token_amount,
                info: paired_assets[0].clone(),
            }
        } else {
            return Err(DexError::ArgumentMismatch(
                offer_asset.to_string(),
                pair_config
                    .assets
                    .iter()
                    .map(|e| e.info.to_string())
                    .collect(),
            ));
        };

        let offer_assets = [offer_asset, other_asset];

        let coins = coins_in_assets(&offer_assets);
        // approval msgs for cw20 tokens (if present)
        let mut msgs = cw_approve_msgs(&offer_assets, &pair_address)?;

        // construct execute msg
        let terraswap_assets = offer_assets
            .iter()
            .map(cw_asset_to_terraswap)
            .collect::<Result<Vec<_>, _>>()?;
        let msg = terraswap::pair::ExecuteMsg::ProvideLiquidity {
            assets: [terraswap_assets[0].clone(), terraswap_assets[1].clone()],
            slippage_tolerance: None,
            receiver: None,
        };
        // actual call to pair
        let liquidity_msg = wasm_execute(pair_address, &msg, coins)?.into();
        msgs.push(liquidity_msg);

        Ok(msgs)
    }

    fn withdraw_liquidity(
        &self,
        _deps: Deps,
        pool_id: PoolAddress,
        lp_token: Asset,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pair_address = pool_id.expect_contract()?;
        let hook_msg = terraswap::pair::Cw20HookMsg::WithdrawLiquidity {};
        // Call swap on pair through cw20 Send
        let withdraw_msg = lp_token.send_msg(pair_address, to_binary(&hook_msg)?)?;

        Ok(vec![withdraw_msg])
    }

    fn simulate_swap(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        _ask_asset: AssetInfo,
    ) -> Result<(Return, Spread, Fee, FeeOnInput), DexError> {
        let pair_address = pool_id.expect_contract()?;
        // Do simulation
        let SimulationResponse {
            return_amount,
            spread_amount,
            commission_amount,
        } = deps.querier.query(&wasm_smart_query(
            pair_address.to_string(),
            &terraswap::pair::QueryMsg::Simulation {
                offer_asset: cw_asset_to_terraswap(&offer_asset)?,
            },
        )?)?;
        // commission paid in result asset
        Ok((return_amount, spread_amount, commission_amount, false))
    }
}

fn cw_asset_to_terraswap(asset: &Asset) -> Result<terraswap::asset::Asset, DexError> {
    match &asset.info {
        AssetInfoBase::Native(denom) => Ok(terraswap::asset::Asset {
            amount: asset.amount,
            info: terraswap::asset::AssetInfo::NativeToken {
                denom: denom.clone(),
            },
        }),
        AssetInfoBase::Cw20(contract_addr) => Ok(terraswap::asset::Asset {
            amount: asset.amount,
            info: terraswap::asset::AssetInfo::Token {
                contract_addr: contract_addr.to_string(),
            },
        }),
        _ => Err(DexError::UnsupportedAssetType(asset.info.to_string())),
    }
}
