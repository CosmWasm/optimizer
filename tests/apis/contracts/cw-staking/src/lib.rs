pub mod contract;
pub mod error;
pub mod msg;
mod providers;

mod handlers;
mod traits;

pub use traits::cw_staking_adapter::CwStakingAdapter;
pub use traits::local_cw_staking::LocalCwStaking;

pub const CW_STAKING: &str = "abstract:cw-staking";

#[cfg(any(feature = "juno", feature = "osmosis"))]
pub mod host_staking {
    pub use super::providers::osmosis::Osmosis;
}

#[cfg(feature = "boot")]
pub mod boot {
    use crate::msg::{CwStakingAction, CwStakingExecuteMsg, ExecuteMsg, InstantiateMsg, QueryMsg};
    use crate::CW_STAKING;
    use abstract_boot::boot_core::ContractWrapper;
    use abstract_boot::boot_core::{contract, ContractInstance};
    use abstract_boot::boot_core::{Contract, CwEnv, IndexResponse, TxResponse};
    use abstract_boot::{AbstractBootError, ApiDeployer, Manager};
    use abstract_core::objects::AnsAsset;
    use abstract_core::{api, MANAGER};
    use cosmwasm_std::{Addr, Empty};

    /// Contract wrapper for interacting with BOOT
    #[contract(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
    pub struct CwStakingApi;

    impl<Chain: CwEnv> ApiDeployer<Chain, Empty> for CwStakingApi<Chain> {}

    /// implement chain-generic functions
    impl<Chain: CwEnv> CwStakingApi<Chain>
    where
        TxResponse<Chain>: IndexResponse,
    {
        pub fn new(id: &str, chain: Chain) -> Self {
            Self(
                Contract::new(id, chain)
                    .with_wasm_path("abstract_cw_staking_api")
                    .with_mock(Box::new(ContractWrapper::new_with_empty(
                        crate::contract::execute,
                        crate::contract::instantiate,
                        crate::contract::query,
                    ))),
            )
        }

        pub fn load(chain: Chain, addr: &Addr) -> Self {
            Self(Contract::new(CW_STAKING, chain).with_address(Some(addr)))
        }

        /// Swap using Abstract's OS (registered in daemon_state).
        pub fn stake(
            &self,
            stake_asset: AnsAsset,
            provider: String,
            duration: Option<cw_utils::Duration>,
        ) -> Result<(), AbstractBootError> {
            let manager = Manager::new(MANAGER, self.get_chain().clone());
            let stake_msg = crate::msg::ExecuteMsg::Module(api::ApiRequestMsg {
                proxy_address: None,
                request: CwStakingExecuteMsg {
                    provider,
                    action: CwStakingAction::Stake {
                        staking_token: stake_asset,
                        unbonding_period: duration,
                    },
                },
            });
            manager.execute_on_module(CW_STAKING, stake_msg)?;
            Ok(())
        }

        pub fn unstake(
            &self,
            stake_asset: AnsAsset,
            provider: String,
            duration: Option<cw_utils::Duration>,
        ) -> Result<(), AbstractBootError> {
            let manager = Manager::new(MANAGER, self.get_chain().clone());
            let stake_msg = crate::msg::ExecuteMsg::Module(api::ApiRequestMsg {
                proxy_address: None,
                request: CwStakingExecuteMsg {
                    provider,
                    action: CwStakingAction::Unstake {
                        staking_token: stake_asset,
                        unbonding_period: duration,
                    },
                },
            });
            manager.execute_on_module(CW_STAKING, stake_msg)?;
            Ok(())
        }
    }
}
