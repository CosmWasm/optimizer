use abstract_api::ApiError;
use abstract_sdk::AbstractSdkError;
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TendermintStakeError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    ApiError(#[from] ApiError),
}
