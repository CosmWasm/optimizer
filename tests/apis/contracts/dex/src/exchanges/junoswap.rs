use crate::{
    commands::{coins_in_assets, cw_approve_msgs},
    dex_trait::{Fee, FeeOnInput, Return, Spread},
};
use crate::{dex_trait::Identify, error::DexError, DEX};
use abstract_core::objects::PoolAddress;
use abstract_sdk::cw_helpers::cosmwasm_std::wasm_smart_query;
use cosmwasm_std::{
    to_binary, wasm_execute, Coin, CosmosMsg, Decimal, Deps, Fraction, Uint128, WasmMsg,
};
use cw20_junoswap::{Cw20ExecuteMsg, Denom};
use cw_asset::{Asset, AssetInfo, AssetInfoBase};
use wasmswap::msg::*;

pub const JUNOSWAP: &str = "junoswap";

// Source https://github.com/wasmswap/wasmswap-contracts
pub struct JunoSwap {}

impl Identify for JunoSwap {
    fn name(&self) -> &'static str {
        JUNOSWAP
    }
    fn over_ibc(&self) -> bool {
        false
    }
}

impl DEX for JunoSwap {
    fn swap(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        ask_asset: AssetInfo,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pair_address = pool_id.expect_contract()?;

        let pair_config: InfoResponse = deps.querier.query(&wasm_smart_query(
            pair_address.to_string(),
            &QueryMsg::Info {},
        )?)?;

        let (offer_token, price) =
            if denom_and_asset_match(&pair_config.token1_denom, &offer_asset.info)? {
                (
                    TokenSelect::Token1,
                    Decimal::from_ratio(pair_config.token2_reserve, pair_config.token1_reserve),
                )
            } else if denom_and_asset_match(&pair_config.token1_denom, &ask_asset)? {
                (
                    TokenSelect::Token2,
                    Decimal::from_ratio(pair_config.token1_reserve, pair_config.token2_reserve),
                )
            } else {
                return Err(DexError::DexMismatch(
                    format!("{}/{}", &offer_asset.info, &ask_asset),
                    self.name().into(),
                    pair_address.to_string(),
                ));
            };

        let min_out: Uint128 = match max_spread {
            None => 0u128.into(),
            Some(spread) => {
                let price_to_use = belief_price.unwrap_or(price);
                let ideal_return = offer_asset.amount * price_to_use;
                ideal_return * (Decimal::one() - spread)
            }
        };

        let swap_msg = ExecuteMsg::Swap {
            input_token: offer_token,
            input_amount: offer_asset.amount,
            min_output: min_out,
            expiration: None,
        };
        let msgs = match &offer_asset.info {
            AssetInfoBase::Cw20(token_addr) => {
                let allowance_msg = wasm_execute(
                    token_addr.to_string(),
                    &Cw20ExecuteMsg::IncreaseAllowance {
                        spender: pair_address.to_string(),
                        amount: offer_asset.amount,
                        expires: None,
                    },
                    vec![],
                )?
                .into();
                let swap_msg = wasm_execute(pair_address, &swap_msg, vec![])?;
                Ok(vec![allowance_msg, swap_msg.into()])
            }
            AssetInfoBase::Native(denom) => Ok(vec![wasm_execute(
                pair_address,
                &swap_msg,
                vec![Coin::new(offer_asset.amount.u128(), denom)],
            )?
            .into()]),
            _ => Err(DexError::UnsupportedAssetType(offer_asset.info.to_string())),
        }?;
        Ok(msgs)
    }

    fn provide_liquidity(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_assets: Vec<Asset>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pair_address = pool_id.expect_contract()?;
        if offer_assets.len() > 2 {
            return Err(DexError::TooManyAssets(2));
        }
        let pair_config: InfoResponse = deps.querier.query(&wasm_smart_query(
            pair_address.to_string(),
            &QueryMsg::Info {},
        )?)?;
        let (token1, token2) =
            if denom_and_asset_match(&pair_config.token1_denom, &offer_assets[0].info)? {
                (&offer_assets[0], &offer_assets[1])
            } else if denom_and_asset_match(&pair_config.token1_denom, &offer_assets[1].info)? {
                (&offer_assets[1], &offer_assets[0])
            } else {
                return Err(DexError::DexMismatch(
                    format!("{}/{}", offer_assets[0].info, offer_assets[1].info),
                    self.name().into(),
                    pair_address.to_string(),
                ));
            };

        let my_ratio = Decimal::from_ratio(token1.amount, token2.amount);
        let max_token2 = if let Some(max_spread) = max_spread {
            token1.amount * my_ratio.inv().unwrap() * (max_spread + Decimal::one())
        } else {
            Uint128::MAX
        };

        let msg = ExecuteMsg::AddLiquidity {
            token1_amount: token1.amount,
            min_liquidity: Uint128::zero(),
            max_token2,
            expiration: None,
        };
        let mut msgs = cw_approve_msgs(&offer_assets, &pair_address)?;
        let coins = coins_in_assets(&offer_assets);
        let junoswap_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: pair_address.into_string(),
            msg: to_binary(&msg)?,
            funds: coins,
        });
        msgs.push(junoswap_msg);
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
        let pair_config: InfoResponse = deps.querier.query(&wasm_smart_query(
            pair_address.to_string(),
            &QueryMsg::Info {},
        )?)?;
        // because of the token1 / token2 thing we need to figure out what the offer asset is and calculate the required amount of the other asset.
        let (token_1_amount, token_2_amount, other_asset) =
            if denom_and_asset_match(&pair_config.token1_denom, &offer_asset.info)? {
                let price =
                    Decimal::from_ratio(pair_config.token2_reserve, pair_config.token1_reserve);
                // token2 = token1 * (token2/token1)
                let token_2_amount = offer_asset.amount * price;
                let other_asset = Asset {
                    info: paired_assets[0].clone(),
                    amount: token_2_amount,
                };
                (offer_asset.amount, token_2_amount, other_asset)
            } else if denom_and_asset_match(&pair_config.token2_denom, &offer_asset.info)? {
                let price =
                    Decimal::from_ratio(pair_config.token1_reserve, pair_config.token2_reserve);
                // token1 = token2 * (token1/token2)
                let token_1_amount = offer_asset.amount * price;
                let other_asset = Asset {
                    info: paired_assets[0].clone(),
                    amount: token_1_amount,
                };
                (token_1_amount, offer_asset.amount, other_asset)
            } else {
                return Err(DexError::DexMismatch(
                    format!("{}/{}", offer_asset.info, paired_assets[0]),
                    self.name().into(),
                    pair_address.to_string(),
                ));
            };

        let msg = ExecuteMsg::AddLiquidity {
            token1_amount: token_1_amount,
            min_liquidity: Uint128::zero(),
            max_token2: token_2_amount,
            expiration: None,
        };
        let assets = &[offer_asset, other_asset];
        let mut msgs = cw_approve_msgs(assets, &pair_address)?;
        let coins = coins_in_assets(assets);
        let junoswap_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: pair_address.into_string(),
            msg: to_binary(&msg)?,
            funds: coins,
        });
        msgs.push(junoswap_msg);
        Ok(msgs)
    }

    fn withdraw_liquidity(
        &self,
        _deps: Deps,
        pool_id: PoolAddress,
        lp_token: Asset,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pair_address = pool_id.expect_contract()?;
        // approve lp token spend
        let mut msgs = cw_approve_msgs(&[lp_token.clone()], &pair_address)?;
        // dex msg
        let junoswap_msg = wasm_execute(
            pair_address,
            &ExecuteMsg::RemoveLiquidity {
                amount: lp_token.amount,
                min_token1: Uint128::zero(),
                min_token2: Uint128::zero(),
                expiration: None,
            },
            vec![],
        )?;
        msgs.push(junoswap_msg.into());
        Ok(msgs)
    }

    fn simulate_swap(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        ask_asset: AssetInfo,
    ) -> Result<(Return, Spread, Fee, FeeOnInput), DexError> {
        let pair_address = pool_id.expect_contract()?;

        let pair_config: InfoResponse = deps.querier.query(&wasm_smart_query(
            pair_address.to_string(),
            &QueryMsg::Info {},
        )?)?;

        let (return_amount, spread_amount) =
            if denom_and_asset_match(&pair_config.token1_denom, &offer_asset.info)? {
                let price =
                    Decimal::from_ratio(pair_config.token2_reserve, pair_config.token1_reserve);
                let ideal_return = offer_asset.amount * price;

                let sim_resp: Token1ForToken2PriceResponse = deps.querier.query_wasm_smart(
                    pair_address,
                    &QueryMsg::Token1ForToken2Price {
                        token1_amount: offer_asset.amount,
                    },
                )?;
                let spread = ideal_return - sim_resp.token2_amount;
                (sim_resp.token2_amount, spread)
            } else if denom_and_asset_match(&pair_config.token1_denom, &ask_asset)? {
                let price =
                    Decimal::from_ratio(pair_config.token1_reserve, pair_config.token2_reserve);
                let ideal_return = offer_asset.amount * price;

                let sim_resp: Token2ForToken1PriceResponse = deps.querier.query_wasm_smart(
                    pair_address,
                    &QueryMsg::Token2ForToken1Price {
                        token2_amount: offer_asset.amount,
                    },
                )?;
                let spread = ideal_return - sim_resp.token1_amount;

                (sim_resp.token1_amount, spread)
            } else {
                return Err(DexError::DexMismatch(
                    format!("{}/{}", &offer_asset.info, &ask_asset),
                    self.name().into(),
                    pair_address.to_string(),
                ));
            };
        Ok((return_amount, spread_amount, Uint128::zero(), true))
    }
}

fn denom_and_asset_match(denom: &Denom, asset: &AssetInfo) -> Result<bool, DexError> {
    match denom {
        Denom::Native(denom_name) => match asset {
            cw_asset::AssetInfoBase::Native(asset_name) => Ok(denom_name == asset_name),
            cw_asset::AssetInfoBase::Cw20(_asset_addr) => Ok(false),
            _ => Err(DexError::UnsupportedAssetType(asset.to_string())),
        },
        Denom::Cw20(denom_addr) => match asset {
            cw_asset::AssetInfoBase::Native(_asset_name) => Ok(false),
            cw_asset::AssetInfoBase::Cw20(asset_addr) => Ok(denom_addr == asset_addr),
            _ => Err(DexError::UnsupportedAssetType(asset.to_string())),
        },
    }
}
