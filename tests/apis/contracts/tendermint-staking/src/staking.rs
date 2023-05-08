//! # Staking
use cosmwasm_std::{
    Addr, Api, Coin, CosmosMsg, DistributionMsg, QuerierWrapper, StakingMsg, StdError, StdResult,
};

pub fn delegate_to(
    querier: &QuerierWrapper,
    validator: &str,
    amount: u128,
) -> StdResult<CosmosMsg> {
    let denom = querier.query_bonded_denom()?;
    Ok(CosmosMsg::Staking(StakingMsg::Delegate {
        validator: validator.to_string(),
        amount: Coin::new(amount, denom),
    }))
}

pub fn undelegate_from(
    querier: &QuerierWrapper,
    validator: &str,
    amount: u128,
) -> StdResult<CosmosMsg> {
    let denom = querier.query_bonded_denom()?;
    Ok(CosmosMsg::Staking(StakingMsg::Undelegate {
        validator: validator.to_string(),
        amount: Coin::new(amount, denom),
    }))
}

pub fn redelegate(
    querier: &QuerierWrapper,
    source_validator: &str,
    destination_validator: &str,
    amount: u128,
) -> StdResult<CosmosMsg> {
    let denom = querier.query_bonded_denom()?;

    Ok(CosmosMsg::Staking(StakingMsg::Redelegate {
        src_validator: source_validator.to_string(),
        dst_validator: destination_validator.to_string(),
        amount: Coin::new(amount, denom),
    }))
}

pub fn redelegate_all(
    querier: &QuerierWrapper,
    source_validator: &str,
    destination_validator: &str,
    proxy_address: &Addr,
) -> StdResult<CosmosMsg> {
    let delegation = querier
        .query_delegation(proxy_address, source_validator)?
        .ok_or(StdError::GenericErr {
            msg: format!("OS not delegated to validator {source_validator}"),
        })?;
    Ok(CosmosMsg::Staking(StakingMsg::Redelegate {
        src_validator: source_validator.to_string(),
        dst_validator: destination_validator.to_string(),
        amount: delegation.amount,
    }))
}

pub fn update_withdraw_address(api: &dyn Api, new_withdraw_address: &str) -> StdResult<CosmosMsg> {
    api.addr_validate(new_withdraw_address)?;
    Ok(CosmosMsg::Distribution(
        DistributionMsg::SetWithdrawAddress {
            address: new_withdraw_address.to_string(),
        },
    ))
}

pub fn withdraw_rewards(validator: &str) -> CosmosMsg {
    CosmosMsg::Distribution(DistributionMsg::WithdrawDelegatorReward {
        validator: validator.to_string(),
    })
}

pub fn withdraw_all_rewards(
    querier: &QuerierWrapper,
    proxy_address: &Addr,
) -> StdResult<Vec<CosmosMsg>> {
    let delegations = querier.query_all_delegations(proxy_address)?;
    let claim_msgs = delegations
        .into_iter()
        .map(|delegation| {
            CosmosMsg::Distribution(DistributionMsg::WithdrawDelegatorReward {
                validator: delegation.validator,
            })
        })
        .collect();
    Ok(claim_msgs)
}

pub fn undelegate_all_from(
    querier: &QuerierWrapper,
    proxy_address: &Addr,
    validator: &str,
) -> StdResult<CosmosMsg> {
    let delegation =
        querier
            .query_delegation(proxy_address, validator)?
            .ok_or(StdError::GenericErr {
                msg: format!("OS not delegated to validator {validator}"),
            })?;
    Ok(CosmosMsg::Staking(StakingMsg::Undelegate {
        validator: validator.to_string(),
        amount: delegation.amount,
    }))
}

pub fn undelegate_all(querier: &QuerierWrapper, proxy_address: &Addr) -> StdResult<Vec<CosmosMsg>> {
    let delegations = querier.query_all_delegations(proxy_address)?;
    let undelegate_msgs = delegations
        .into_iter()
        .map(|delegation| {
            CosmosMsg::Staking(StakingMsg::Undelegate {
                validator: delegation.validator,
                amount: delegation.amount,
            })
        })
        .collect();
    Ok(undelegate_msgs)
}
