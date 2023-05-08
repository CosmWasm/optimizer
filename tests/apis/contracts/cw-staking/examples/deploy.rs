use abstract_boot::boot_core::{
    instantiate_daemon_env, networks::NetworkInfo, DaemonOptionsBuilder, *,
};
use abstract_boot::{AnsHost, ApiDeployer, VCExecFns, VersionControl};
use abstract_cw_staking_api::boot::CwStakingApi;
use abstract_cw_staking_api::CW_STAKING;
use abstract_sdk::core::objects::module::{Module, ModuleInfo, ModuleVersion};
use abstract_sdk::core::{api, ANS_HOST, VERSION_CONTROL};
use cosmwasm_std::{Addr, Empty};
use semver::Version;
use std::sync::Arc;
use tokio::runtime::Runtime;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn deploy_cw_staking(
    network: NetworkInfo,
    prev_version: Option<String>,
    code_id: Option<u64>,
) -> anyhow::Result<()> {
    let module_version: Version = CONTRACT_VERSION.parse().unwrap();

    let rt = Arc::new(Runtime::new()?);
    let options = DaemonOptionsBuilder::default().network(network).build();
    let (_sender, chain) = instantiate_daemon_env(&rt, options?)?;

    let version_control = VersionControl::new(VERSION_CONTROL, chain.clone());
    version_control.set_address(&Addr::unchecked(
        std::env::var("VERSION_CONTROL").expect("VERSION_CONTROL not set"),
    ));

    let ans_host = AnsHost::new(ANS_HOST, chain.clone());

    if let Some(prev_version) = prev_version {
        let Module { info, reference } = version_control.module(ModuleInfo::from_id(
            CW_STAKING,
            ModuleVersion::from(prev_version),
        )?)?;

        let new_info = ModuleInfo {
            version: ModuleVersion::from(CONTRACT_VERSION),
            ..info
        };
        version_control.add_modules(vec![(new_info, reference)])?;
    } else if let Some(code_id) = code_id {
        let mut cw_staking = CwStakingApi::new(CW_STAKING, chain);
        cw_staking.set_code_id(code_id);
        let init_msg = api::InstantiateMsg {
            module: Empty {},
            base: api::BaseInstantiateMsg {
                ans_host_address: ans_host.address()?.into(),
                version_control_address: version_control.address()?.into(),
            },
        };
        cw_staking
            .as_instance_mut()
            .instantiate(&init_msg, None, None)?;

        version_control.register_apis(vec![cw_staking.as_instance_mut()], &module_version)?;
    } else {
        log::info!("Uploading Cw staking");
        // Upload and deploy with the version
        let mut cw_staking = CwStakingApi::new(CW_STAKING, chain);

        cw_staking.deploy(module_version, Empty {})?;
    }

    Ok(())
}

use clap::Parser;

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Use a previously deployed version instead of uploading the new one
    #[arg(short, long)]
    prev_version: Option<String>,
    #[arg(short, long)]
    network_id: String,
    #[arg(short, long)]
    code_id: Option<u64>,
}

fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    let Arguments {
        network_id,
        prev_version,
        code_id,
    } = Arguments::parse();

    let network = boot_core::networks::parse_network(&network_id);

    deploy_cw_staking(network, prev_version, code_id)
}
