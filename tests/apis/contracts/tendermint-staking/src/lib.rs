pub mod contract;
pub mod error;
pub mod msg;
mod staking;

pub const TENDERMINT_STAKING: &str = "abstract:tendermint-staking";

#[cfg(feature = "boot")]
pub mod boot {
    use abstract_boot::boot_core::ContractWrapper;
    use abstract_boot::boot_core::{contract, Contract, CwEnv};
    use abstract_boot::ApiDeployer;
    use abstract_core::api::InstantiateMsg;
    use cosmwasm_std::Empty;

    use crate::msg::*;

    #[contract(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
    pub struct TMintStakingApi<Chain>;

    impl<Chain: CwEnv> ApiDeployer<Chain, Empty> for TMintStakingApi<Chain> {}

    impl<Chain: CwEnv> TMintStakingApi<Chain> {
        pub fn new(name: &str, chain: Chain) -> Self {
            Self(
                Contract::new(name, chain)
                    .with_wasm_path("abstract_tendermint_staking_api")
                    .with_mock(Box::new(ContractWrapper::new_with_empty(
                        crate::contract::execute,
                        crate::contract::instantiate,
                        crate::contract::query,
                    ))),
            )
        }
    }
}
