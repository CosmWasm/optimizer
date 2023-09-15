use crate::{contract, is_cargo_project, WorkspaceCargoToml};
use std::fs;

const PACKAGE_PREFIX: &str = "contracts/";

pub fn build_workspace() {
    let file = fs::read_to_string("Cargo.toml").unwrap();
    let cargo_toml: WorkspaceCargoToml = toml::from_str(&file).unwrap();
    let members = cargo_toml.workspace.members;

    println!("Found workspace member entries: {:?}", members);

    let mut all_packages = members
        .iter()
        .map(|member| {
            glob::glob(member)
                .unwrap()
                .map(|path| path.unwrap())
                .filter(is_cargo_project)
        })
        .flatten()
        .collect::<Vec<_>>();

    all_packages.sort();

    println!("Package directories: {:?}", all_packages);

    let contract_packages = all_packages
        .iter()
        .filter(|p| p.starts_with(PACKAGE_PREFIX))
        .collect::<Vec<_>>();

    println!("Contracts to be built: {:?}", contract_packages);

    for contract in contract_packages {
        // Build all the contracts and their features
        contract::build_contract(contract);
    }
}
