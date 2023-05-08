use crate::contract::CwStakingResult;
use crate::msg::{Claim, StakeResponse, StakingInfoResponse, UnbondingResponse};
use crate::traits::cw_staking_adapter::CwStakingAdapter;
use crate::traits::identify::Identify;
use crate::{error::StakingError, msg::RewardTokensResponse};
use abstract_sdk::{
    core::objects::{AssetEntry, LpToken},
    feature_objects::AnsHost,
    AbstractSdkError, Resolve,
};
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, Deps, Env, QuerierWrapper, StdError, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use cw20_stake::msg::{ExecuteMsg as StakeCw20ExecuteMsg, ReceiveMsg};
use cw_asset::{AssetInfo, AssetInfoBase};
use cw_utils::Duration;

pub const JUNOSWAP: &str = "junoswap";
// Source https://github.com/wasmswap/wasmswap-contracts
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JunoSwap {
    lp_token: LpToken,
    lp_token_address: Addr,
    staking_contract_address: Addr,
}

impl Default for JunoSwap {
    fn default() -> Self {
        Self {
            lp_token: Default::default(),
            lp_token_address: Addr::unchecked(""),
            staking_contract_address: Addr::unchecked(""),
        }
    }
}

impl Identify for JunoSwap {
    fn name(&self) -> &'static str {
        JUNOSWAP
    }
}

impl CwStakingAdapter for JunoSwap {
    // get the relevant data for Junoswap staking
    fn fetch_data(
        &mut self,
        deps: Deps,
        _env: Env,
        ans_host: &AnsHost,
        lp_token: AssetEntry,
    ) -> Result<(), AbstractSdkError> {
        self.staking_contract_address = self.staking_contract_address(deps, ans_host, &lp_token)?;

        let AssetInfoBase::Cw20(token_addr) = lp_token.resolve(&deps.querier, ans_host)? else {
                return Err(AbstractSdkError::Std(StdError::generic_err("expected CW20 as LP token for staking.")));
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
        let msg = to_binary(&ReceiveMsg::Stake {})?;
        Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.lp_token_address.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: self.staking_contract_address.to_string(),
                amount,
                msg,
            })?,
            funds: vec![],
        })])
    }

    fn unstake(
        &self,
        _deps: Deps,
        amount: Uint128,
        _unbonding_period: Option<Duration>,
    ) -> Result<Vec<CosmosMsg>, StakingError> {
        let msg = StakeCw20ExecuteMsg::Unstake { amount };
        Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.staking_contract_address.to_string(),
            msg: to_binary(&msg)?,
            funds: vec![],
        })])
    }

    fn claim_rewards(&self, _deps: Deps) -> Result<Vec<CosmosMsg>, StakingError> {
        Ok(vec![])
    }

    fn claim(&self, _deps: Deps) -> Result<Vec<CosmosMsg>, StakingError> {
        let msg = StakeCw20ExecuteMsg::Claim {};

        Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.staking_contract_address.to_string(),
            msg: to_binary(&msg)?,
            funds: vec![],
        })])
    }

    fn query_info(&self, querier: &QuerierWrapper) -> CwStakingResult<StakingInfoResponse> {
        let stake_info_resp: cw20_stake::state::Config = querier.query_wasm_smart(
            self.staking_contract_address.clone(),
            &cw20_stake::msg::QueryMsg::GetConfig {},
        )?;
        Ok(StakingInfoResponse {
            staking_contract_address: self.staking_contract_address.clone(),
            staking_token: AssetInfo::Cw20(stake_info_resp.token_address),
            unbonding_periods: stake_info_resp
                .unstaking_duration
                .map(parse_duration)
                .map(|d| vec![d]),
            max_claims: Some(cw20_stake::state::MAX_CLAIMS as u32),
        })
    }

    fn query_staked(
        &self,
        querier: &QuerierWrapper,
        staker: Addr,
        _unbonding_period: Option<Duration>,
    ) -> CwStakingResult<StakeResponse> {
        let stake_balance: cw20_stake::msg::StakedBalanceAtHeightResponse = querier
            .query_wasm_smart(
                self.staking_contract_address.clone(),
                &cw20_stake::msg::QueryMsg::StakedBalanceAtHeight {
                    address: staker.into_string(),
                    height: None,
                },
            )?;
        Ok(StakeResponse {
            amount: stake_balance.balance,
        })
    }

    fn query_unbonding(
        &self,
        querier: &QuerierWrapper,
        staker: Addr,
    ) -> CwStakingResult<UnbondingResponse> {
        let claims: cw20_stake::msg::ClaimsResponse = querier.query_wasm_smart(
            self.staking_contract_address.clone(),
            &cw20_stake::msg::QueryMsg::Claims {
                address: staker.into_string(),
            },
        )?;
        let claims = claims
            .claims
            .iter()
            .map(|claim| Claim {
                amount: claim.amount,
                claimable_at: parse_expiration(claim.release_at),
            })
            .collect();
        Ok(UnbondingResponse { claims })
    }
    fn query_reward_tokens(
        &self,
        _querier: &QuerierWrapper,
    ) -> CwStakingResult<RewardTokensResponse> {
        todo!()
    }
}

fn parse_duration(d: dao_cw_utils::Duration) -> cw_utils::Duration {
    match d {
        dao_cw_utils::Duration::Height(a) => cw_utils::Duration::Height(a),
        dao_cw_utils::Duration::Time(a) => cw_utils::Duration::Time(a),
    }
}

fn parse_expiration(d: dao_cw_utils::Expiration) -> cw_utils::Expiration {
    match d {
        dao_cw_utils::Expiration::AtHeight(a) => cw_utils::Expiration::AtHeight(a),
        dao_cw_utils::Expiration::AtTime(a) => cw_utils::Expiration::AtTime(a),
        _ => cw_utils::Expiration::Never {},
    }
}
