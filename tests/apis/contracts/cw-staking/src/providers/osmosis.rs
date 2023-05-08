use cosmwasm_std::Addr;

use crate::traits::identify::Identify;

pub const OSMOSIS: &str = "osmosis";

#[derive(Default)]
pub struct Osmosis {
    pub local_proxy_addr: Option<Addr>,
}

impl Identify for Osmosis {
    fn name(&self) -> &'static str {
        OSMOSIS
    }
}
#[cfg(feature = "osmosis")]
pub mod fns {
    use crate::CwStakingAdapter;

    use super::*;
    const FORTEEN_DAYS: i64 = 60 * 60 * 24 * 14;
    use abstract_core::objects::LpToken;
    use abstract_sdk::Resolve;
    use cosmwasm_std::Env;
    use osmosis_std::{
        shim::Duration,
        types::osmosis::gamm::v1beta1::{
            MsgExitPool, MsgJoinPool, MsgSwapExactAmountIn, QuerySwapExactAmountInRequest,
            SwapAmountInRoute,
        },
        types::{
            cosmos::base::v1beta1::Coin as OsmoCoin,
            osmosis::gamm::v1beta1::{Pool, QueryPoolRequest},
        },
        types::{osmosis::lockup::MsgBeginUnlocking, osmosis::lockup::MsgLockTokens},
    };

    /// Osmosis app-chain dex implementation
    impl CwStakingAdapter for Osmosis {
        fn fetch_data(
            &mut self,
            deps: cosmwasm_std::Deps,
            env: Env,
            ans_host: &abstract_sdk::feature_objects::AnsHost,
            staking_asset: abstract_core::objects::AssetEntry,
        ) -> abstract_sdk::AbstractSdkResult<()> {
            let provider_staking_contract_entry = self.staking_entry(&staking_asset);

            // self.local_proxy_addr = Some("something_else");

            let pool_addr =
                ans_host.query_contract(&deps.querier, &provider_staking_contract_entry)?;
            // TODO: How is the staking address the local proxy address?

            // TODO: this is the validator address
            self.validator_addr = Some(Addr::unchecked(
                "osmovaloper1v9w0j3x7q5yqy0y3q3x6y2z7xqz3zq5q9zq3zq",
            ));

            // TODO: this is the superfluid lock id which 'i think' is generated at first lock
            // We need a way to initialize that lock id and store it...
            self.sf_lock_id = Some(10);

            // TODO: this is the gamm pool address/number. Im not sure how this will be read/stored
            self.pool_addr = Some(pool_addr); // Some(Addr::unchecked("gamm/pool/69"));
                                              // TODO: this is the gamm pool id. Im not sure how this will be read/stored
            self.pool_id = Some(69);
            Ok(())
        }

        fn stake(
            &self,
            deps: Deps,
            amount: cosmwasm_std::Uint128,
            unbonding_period: Option<cw_utils::Duration>,
        ) -> Result<Vec<cosmwasm_std::CosmosMsg>, StakingError> {
            // Query lock id from the proxy address (local proxy address)
            let proxy_address = self.local_proxy_addr.as_ref().unwrap();
            let validator = self.validator_addr.as_ref().unwrap();

            deps.querier.query_delegation(proxy_address, validator);
            deps.querier.query_all_delegations(proxy_address);

            let coin = OsmoCoin {
                // NOTE: This shold be the gamm token address
                denom: self.pool_addr.as_ref().unwrap().to_string(),
                amount: amount.to_string(),
            };

            let msg: CosmosMsg = MsgLockTokens {
                duration: Some(Duration {
                    seconds: FORTEEN_DAYS,
                    nanos: 0,
                }),
                owner: self.local_proxy_addr.as_ref().unwrap().to_string(),
                coins: vec![coin],
            }
            .into();

            Ok(vec![msg])
        }

        fn unstake(
            &self,
            deps: Deps,
            staking_address: Addr,
            amount: Asset,
        ) -> Result<Vec<CosmosMsg>, StakingError> {
            let gamm_id = 1; // TODO: Resolve the right gamm id and add it here

            let msg: CosmosMsg = MsgBeginUnlocking {
                owner: self.local_proxy_addr.as_ref().unwrap().to_string(),
                coins: vec![OsmoCoin::try_from(amount).unwrap()], // see docs: "Unlocking all if unset"
                id: gamm_id,
            }
            .into();

            Ok(vec![msg])
        }

        fn claim(&self, deps: Deps, staking_address: Addr) -> Result<Vec<CosmosMsg>, StakingError> {
            unimplemented!()
        }

        fn claim_rewards(&self, deps: Deps) -> Result<Vec<cosmwasm_std::CosmosMsg>, StakingError> {
            Ok(vec![])
        }

        // fn query_pool_data(
        //     &self,
        //     querier: &cosmwasm_std::QuerierWrapper,
        //     pool_id: u64,
        // ) -> StdResult<Pool> {
        //     let res = QueryPoolRequest { pool_id }.query(&querier).unwrap();

        //     let pool = Pool::try_from(res.pool.unwrap()).unwrap();
        //     Ok(pool)
        // }

        fn query_info(
            &self,
            querier: &cosmwasm_std::QuerierWrapper,
        ) -> crate::contract::CwStakingResult<crate::msg::StakingInfoResponse> {
            let pool = self
                .query_pool_data(querier, self.pool_id.unwrap())
                .unwrap();

            let res = StakingInfoResponse {
                staking_token: AssetInfoBase::Cw20(Addr::unchecked(
                    self.pool_addr.as_ref().unwrap().to_string(),
                )),
                staking_contract_address: self.pool_addr.clone().unwrap(),
                unbonding_periods: Some(vec![]),
                max_claims: None,
            };

            Ok(res)
        }

        fn query_pool_data(deps: Deps, pool_id: u64) -> StdResult<Pool> {
            let res = QueryPoolRequest { pool_id }.query(&deps.querier).unwrap();

            let pool = Pool::try_from(res.pool.unwrap()).unwrap();
            Ok(pool)
        }

        fn compute_osmo_share_out_amount(
            pool_assets: &[OsmoCoin],
            deposits: &[Uint128; 2],
            total_share: Uint128,
        ) -> StdResult<Uint128> {
            // ~ source: terraswap contract ~
            // min(1, 2)
            // 1. sqrt(deposit_0 * exchange_rate_0_to_1 * deposit_0) * (total_share / sqrt(pool_0 * pool_1))
            // == deposit_0 * total_share / pool_0
            // 2. sqrt(deposit_1 * exchange_rate_1_to_0 * deposit_1) * (total_share / sqrt(pool_1 * pool_1))
            // == deposit_1 * total_share / pool_1
            let share_amount_out = std::cmp::min(
                deposits[0].multiply_ratio(
                    total_share,
                    pool_assets[0].amount.parse::<Uint128>().unwrap(),
                ),
                deposits[1].multiply_ratio(
                    total_share,
                    pool_assets[1].amount.parse::<Uint128>().unwrap(),
                ),
            );

            Ok(share_amount_out)
        }

        fn assert_slippage_tolerance(
            slippage_tolerance: &Option<Decimal>,
            deposits: &[Uint128; 2],
            pool_assets: &[OsmoCoin],
        ) -> Result<(), StakingError> {
            if let Some(slippage_tolerance) = *slippage_tolerance {
                let slippage_tolerance: Decimal256 = slippage_tolerance.into();
                if slippage_tolerance > Decimal256::one() {
                    return Err(StakingError::Std(StdError::generic_err(
                        "slippage_tolerance cannot bigger than 1",
                    )));
                }

                let one_minus_slippage_tolerance = Decimal256::one() - slippage_tolerance;
                let deposits: [Uint256; 2] = [deposits[0].into(), deposits[1].into()];
                let pools: [Uint256; 2] = [
                    pool_assets[0].amount.parse::<Uint256>().unwrap(),
                    pool_assets[1].amount.parse::<Uint256>().unwrap(),
                ];

                // Ensure each prices are not dropped as much as slippage tolerance rate
                if Decimal256::from_ratio(deposits[0], deposits[1]) * one_minus_slippage_tolerance
                    > Decimal256::from_ratio(pools[0], pools[1])
                    || Decimal256::from_ratio(deposits[1], deposits[0])
                        * one_minus_slippage_tolerance
                        > Decimal256::from_ratio(pools[1], pools[0])
                {
                    return Err(StakingError::MaxSlippageAssertion(
                        slippage_tolerance.to_string(),
                        OSMOSIS.to_owned(),
                    ));
                }
            }

            Ok(())
        }
    }
}
