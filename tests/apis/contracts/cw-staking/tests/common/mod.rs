pub const ROOT_USER: &str = "root_user";
use abstract_boot::AccountFactory;

use abstract_boot::AbstractAccount;
use abstract_core::objects::gov_type::GovernanceDetails;

use abstract_boot::boot_core::Mock;
use cosmwasm_std::Addr;

pub fn create_default_os(factory: &AccountFactory<Mock>) -> anyhow::Result<AbstractAccount<Mock>> {
    let os = factory.create_default_account(GovernanceDetails::Monarchy {
        monarch: Addr::unchecked(ROOT_USER).to_string(),
    })?;
    Ok(os)
}
