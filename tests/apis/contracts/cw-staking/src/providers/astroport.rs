use crate::error::StakingError;
use crate::traits::cw_staking_adapter::CwStakingAdapter;
use crate::traits::identify::Identify;
use abstract_sdk::{
    core::objects::{AssetEntry, LpToken},
    feature_objects::AnsHost,
    AbstractSdkResult, Resolve,
};

#[cfg(feature = "terra")]
use astroport::generator::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg as GeneratorExecuteMsg, QueryMsg as GeneratorQueryMsg,
    RewardInfoResponse,
};

use crate::msg::{RewardTokensResponse, StakeResponse, StakingInfoResponse, UnbondingResponse};
use cosmwasm_std::{
    to_binary, wasm_execute, Addr, CosmosMsg, Deps, Env, QuerierWrapper, StdError, Uint128,
};
use cw20::Cw20ExecuteMsg;
use cw_asset::AssetInfo;
use cw_utils::Duration;

pub const ASTROPORT: &str = "astroport";

// TODO: use optional values here?
#[derive(Clone, Debug)]
pub struct Astroport {
    lp_token: LpToken,
    lp_token_address: Addr,
    generator_contract_address: Addr,
}

impl Default for Astroport {
    fn default() -> Self {
        Self {
            lp_token: Default::default(),
            lp_token_address: Addr::unchecked(""),
            generator_contract_address: Addr::unchecked(""),
        }
    }
}

// Data that's retrieved from ANS
// - LP token address, based on provided LP token
// - Generator address = staking_address
impl Identify for Astroport {
    fn name(&self) -> &'static str {
        ASTROPORT
    }
}

impl CwStakingAdapter for Astroport {
    // get the relevant data for Astroport staking
    fn fetch_data(
        &mut self,
        deps: Deps,
        _env: Env,
        ans_host: &AnsHost,
        lp_token: AssetEntry,
    ) -> AbstractSdkResult<()> {
        self.generator_contract_address =
            self.staking_contract_address(deps, ans_host, &lp_token)?;

        let AssetInfo::Cw20(token_addr) = lp_token.resolve(&deps.querier, ans_host)? else {
                return Err(StdError::generic_err("expected CW20 as LP token for staking.").into());
            };
        self.lp_token_address = token_addr;
        self.lp_token = LpToken::try_from(lp_token)?;
        Ok(())
    }

    fn stake(
        &self,
        _deps: Deps,
        amount: Uint128,
        _unbonding_period: Option<Duration>,
    ) -> Result<Vec<CosmosMsg>, StakingError> {
        let msg = to_binary(&Cw20HookMsg::Deposit {})?;
        Ok(vec![wasm_execute(
            self.lp_token_address.to_string(),
            &Cw20ExecuteMsg::Send {
                contract: self.generator_contract_address.to_string(),
                amount,
                msg,
            },
            vec![],
        )?
        .into()])
    }

    fn unstake(
        &self,
        _deps: Deps,
        amount: Uint128,
        _unbonding_period: Option<Duration>,
    ) -> Result<Vec<CosmosMsg>, StakingError> {
        let msg = GeneratorExecuteMsg::Withdraw {
            lp_token: self.lp_token_address.to_string(),
            amount,
        };
        Ok(vec![wasm_execute(
            self.generator_contract_address.to_string(),
            &msg,
            vec![],
        )?
        .into()])
    }

    fn claim(&self, _deps: Deps) -> Result<Vec<CosmosMsg>, StakingError> {
        Ok(vec![])
    }

    fn claim_rewards(&self, _deps: Deps) -> Result<Vec<CosmosMsg>, StakingError> {
        let msg = GeneratorExecuteMsg::ClaimRewards {
            lp_tokens: vec![self.lp_token_address.clone().into()],
        };

        Ok(vec![wasm_execute(
            self.generator_contract_address.to_string(),
            &msg,
            vec![],
        )?
        .into()])
    }

    fn query_info(&self, querier: &QuerierWrapper) -> Result<StakingInfoResponse, StakingError> {
        let ConfigResponse { astro_token, .. } = querier
            .query_wasm_smart::<ConfigResponse>(
                self.generator_contract_address.clone(),
                &GeneratorQueryMsg::Config {},
            )
            .map_err(|e| {
                StdError::generic_err(format!(
                    "Failed to query staking info for {} with generator: {}, {:?}",
                    self.name(),
                    self.generator_contract_address.clone(),
                    e
                ))
            })?;

        let astro_token = AssetInfo::cw20(astro_token);

        Ok(StakingInfoResponse {
            staking_contract_address: self.generator_contract_address.clone(),
            staking_token: astro_token,
            unbonding_periods: None,
            max_claims: None,
        })
    }

    fn query_staked(
        &self,
        querier: &QuerierWrapper,
        staker: Addr,
        _unbonding_period: Option<Duration>,
    ) -> Result<StakeResponse, StakingError> {
        let stake_balance: Uint128 = querier
            .query_wasm_smart(
                self.generator_contract_address.clone(),
                &GeneratorQueryMsg::Deposit {
                    lp_token: self.lp_token_address.to_string(),
                    user: staker.to_string(),
                },
            )
            .map_err(|e| {
                StdError::generic_err(format!(
                    "Failed to query staked balance on {} for {}. Error: {:?}",
                    self.name(),
                    staker,
                    e
                ))
            })?;
        Ok(StakeResponse {
            amount: stake_balance,
        })
    }

    fn query_unbonding(
        &self,
        _querier: &QuerierWrapper,
        _staker: Addr,
    ) -> Result<UnbondingResponse, StakingError> {
        Ok(UnbondingResponse { claims: vec![] })
    }

    fn query_reward_tokens(
        &self,
        querier: &QuerierWrapper,
    ) -> Result<crate::msg::RewardTokensResponse, StakingError> {
        let reward_info: RewardInfoResponse = querier
            .query_wasm_smart(
                self.generator_contract_address.clone(),
                &GeneratorQueryMsg::RewardInfo {
                    lp_token: self.lp_token_address.to_string(),
                },
            )
            .map_err(|e| {
                StdError::generic_err(format!(
                    "Failed to query reward info on {} for lp token {}. Error: {:?}",
                    self.name(),
                    self.lp_token,
                    e
                ))
            })?;

        let mut tokens = { vec![AssetInfo::Cw20(reward_info.base_reward_token)] };

        if let Some(reward_token) = reward_info.proxy_reward_token {
            tokens.push(AssetInfo::cw20(reward_token));
        }
        Ok(RewardTokensResponse { tokens })
    }
}
