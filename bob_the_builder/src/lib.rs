mod cargo_toml;
mod pkg_build;

use glob::glob;
use std::{
    fs::{self},
    path::{Path, PathBuf},
};

use cargo_toml::{
    package::{self},
    workspace::{is_workspace, IsWorkspace},
};

const CARGO_PATH: &str = "cargo";
const PACKAGE_PREFIX: &str = "contracts/";

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

pub fn build() {
    let file = fs::read_to_string("Cargo.toml").unwrap();
    match is_workspace(&file).unwrap() {
        IsWorkspace::Yes { members } => {
            println!("Found workspace member entries: {:?}", &members);
            build_workspace(&members);
        }
        IsWorkspace::NoMembers => {
            println!("Cargo.toml contains a workspace key but has no workspace members");
        }
        IsWorkspace::No => {
            let package = package::parse_toml(&file).unwrap();
            package.build(Path::new("."))
        }
    }
}

pub fn build_workspace(workspace_members: &[String]) {
    let mut all_packages = workspace_members
        .iter()
        .flat_map(|member| {
            glob(member)
                .unwrap()
                .map(|path| path.unwrap())
                .filter(is_cargo_project)
        })
        .collect::<Vec<_>>();

    all_packages.sort();

    println!("Package directories: {:?}", all_packages);

    let contract_packages = all_packages
        .iter()
        .filter(|p| p.starts_with(PACKAGE_PREFIX))
        .collect::<Vec<_>>();

    println!("Contracts to be built: {:?}", contract_packages);

    for contract_dir in contract_packages {
        let contract_cargo_toml = fs::read_to_string(contract_dir.join("Cargo.toml")).unwrap();
        let package = package::parse_toml(&contract_cargo_toml).unwrap();
        println!("Building {:?} ...", package.name);
        package.build(contract_dir);
    }
}
