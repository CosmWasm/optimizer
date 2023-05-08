use abstract_boot::boot_core::{instantiate_default_mock_env, CallAs, ContractInstance, Deploy};
use abstract_boot::{Abstract, ApiDeployer};
use abstract_core::objects::{AnsAsset, AssetEntry};

use boot_cw_plus::Cw20ExecuteMsgFns;
use cosmwasm_std::{Addr, Empty};
use speculoos::*;
use wyndex_bundle::{EUR_USD_LP, WYNDEX, WYNDEX_OWNER};

use abstract_cw_staking_api::CW_STAKING;
use abstract_cw_staking_api::{boot::CwStakingApi, msg::CwStakingQueryMsgFns};
use common::create_default_os;

mod common;

#[test]
fn stake_lp() -> anyhow::Result<()> {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;

    let deployment = Abstract::deploy_on(chain.clone(), "1.0.0".parse()?)?;
    let wyndex = wyndex_bundle::WynDex::store_on(chain.clone())?;

    let _root_os = create_default_os(&deployment.account_factory)?;
    let mut staking_api = CwStakingApi::new(CW_STAKING, chain.clone());

    staking_api.deploy("1.0.0".parse()?, Empty {})?;

    let os = create_default_os(&deployment.account_factory)?;
    let proxy_addr = os.proxy.address()?;
    let _manager_addr = os.manager.address()?;

    // transfer some LP tokens to the AbstractAccount, as if it provided liquidity
    wyndex
        .eur_usd_lp
        .call_as(&Addr::unchecked(WYNDEX_OWNER))
        .transfer(1000u128.into(), proxy_addr.to_string())?;

    // install exchange on AbstractAccount
    os.manager.install_module(CW_STAKING, &Empty {})?;
    // load exchange data into type
    staking_api.set_address(&Addr::unchecked(
        os.manager.module_info(CW_STAKING)?.unwrap().address,
    ));

    let dur = Some(cw_utils::Duration::Time(2));

    // stake 100 EUR
    staking_api.stake(AnsAsset::new(EUR_USD_LP, 100u128), WYNDEX.into(), dur)?;

    // query stake
    let staked_balance = staking_api.staked(
        WYNDEX.into(),
        proxy_addr.to_string(),
        AssetEntry::new(EUR_USD_LP),
        dur,
    )?;
    assert_that!(staked_balance.amount.u128()).is_equal_to(100u128);

    // now unbond 50
    staking_api.unstake(AnsAsset::new(EUR_USD_LP, 50u128), WYNDEX.into(), dur)?;
    // query stake
    let staked_balance = staking_api.staked(
        WYNDEX.into(),
        proxy_addr.to_string(),
        AssetEntry::new(EUR_USD_LP),
        dur,
    )?;
    assert_that!(staked_balance.amount.u128()).is_equal_to(50u128);
    Ok(())
}
