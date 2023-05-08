//! # Tendermint Staking Api
//!
//! `abstract_core::tendermint_staking` exposes all the function of [`cosmwasm_std::CosmosMsg::Staking`] and [`cosmwasm_std::CosmosMsg::Distribution`].

use abstract_core::api;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Uint128;

pub type ExecuteMsg = api::ExecuteMsg<TendermintStakingExecuteMsg>;
pub type QueryMsg = api::QueryMsg<TendermintStakingQueryMsg>;

impl api::ApiExecuteMsg for TendermintStakingExecuteMsg {}
impl api::ApiQueryMsg for TendermintStakingQueryMsg {}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "boot", derive(boot_core::ExecuteFns))]
#[cfg_attr(feature = "boot", impl_into(ExecuteMsg))]
pub enum TendermintStakingExecuteMsg {
    Delegate {
        /// Validator address
        validator: String,
        amount: Uint128,
    },
    UndelegateFrom {
        /// Validator address
        validator: String,
        amount: Option<Uint128>,
    },
    UndelegateAll {},
    Redelegate {
        /// Validator address
        source_validator: String,
        /// Validator address
        destination_validator: String,
        amount: Option<Uint128>,
    },
    SetWithdrawAddress {
        /// The new `withdraw_address`
        new_withdraw_address: String,
    },
    WithdrawDelegatorReward {
        /// Validator address
        validator: String,
    },
    /// Withdraw all the rewards
    WithdrawAllRewards {},
}

/// Staking queries are available on [`cosmwasm_std::QuerierWrapper`] through [`cosmwasm_std::Deps`].
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "boot", derive(boot_core::QueryFns))]
#[cfg_attr(feature = "boot", impl_into(QueryMsg))]
pub enum TendermintStakingQueryMsg {}
