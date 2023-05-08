use crate::msg::{DexApiExecuteMsg, DexInstantiateMsg, DexQueryMsg};
use crate::EXCHANGE;
use crate::{error::DexError, handlers};
use abstract_api::{export_endpoints, ApiContract};
use cosmwasm_std::Response;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type DexApi = ApiContract<DexError, DexInstantiateMsg, DexApiExecuteMsg, DexQueryMsg>;
pub type DexResult<T = Response> = Result<T, DexError>;

pub const DEX_API: DexApi = DexApi::new(EXCHANGE, CONTRACT_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler);

#[cfg(feature = "export")]
export_endpoints!(DEX_API, DexApi);
