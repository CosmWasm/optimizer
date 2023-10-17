use glob::glob;
use serde::Deserialize;
use std::{
    fs::{self, canonicalize},
    path::PathBuf,
    process::Command,
};

const CARGO_PATH: &str = "cargo";
const PACKAGE_PREFIX: &str = "contracts/";

#[derive(Deserialize, Debug)]
pub struct CargoToml {
    workspace: Workspace,
}

#[derive(Deserialize, Debug)]
pub struct Workspace {
    members: Vec<String>,
}

/// Checks if the given path is a Cargo project. This is needed
/// to filter the glob results of a workspace member like `contracts/*`
/// to exclude things like non-directories.
#[allow(clippy::ptr_arg)]
fn is_cargo_project(path: &PathBuf) -> bool {
    // Should we do other checks in here? E.g. filter out hidden directories
    // or directories without Cargo.toml. This should be in line with what
    // cargo does for wildcard workspace members.
    path.is_dir()
}

pub fn build_workspace() {
    let file = fs::read_to_string("Cargo.toml").unwrap();
    let cargo_toml: CargoToml = toml::from_str(&file).unwrap();
    let members = cargo_toml.workspace.members;

    println!("Found workspace member entries: {:?}", members);

    let mut all_packages = members
        .iter()
        .map(|member| {
            glob(member)
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
        println!("Building {:?} ...", contract);

        let mut child = Command::new(CARGO_PATH)
            .args(&[
                "build",
                "--target-dir=/target",
                "--release",
                "--lib",
                "--target=wasm32-unknown-unknown",
                "--locked",
            ])
            .env("RUSTFLAGS", "-C link-arg=-s")
            .current_dir(canonicalize(contract).unwrap())
            .spawn()
            .unwrap();
        let error_code = child.wait().unwrap();
        assert!(error_code.success());
    }
}
