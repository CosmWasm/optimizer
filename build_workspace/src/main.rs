use glob::glob;
use serde::Deserialize;
use std::{fs, process::Command};

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

fn main() {
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
                .collect::<Vec<_>>()
        })
        .flatten()
        .collect::<Vec<_>>();

    all_packages.sort();

    println!("Package directories: {:?}", all_packages);

    let contract_packages = all_packages
        .into_iter()
        .filter(|p| p.starts_with(PACKAGE_PREFIX))
        .collect::<Vec<_>>();

    println!("Contracts to be built: {:?}", contract_packages);

    for contract in contract_packages {
        println!("Building {:?} ...", contract);

        let mut child = Command::new(CARGO_PATH)
            .args(&[
                "build",
                "--release",
                "--target=wasm32-unknown-unknown",
                "--locked",
            ])
            .env("RUSTFLAGS", "-C link-arg=-s")
            .spawn()
            .unwrap();
        let error_code = child.wait().unwrap();
        assert!(error_code.success());
    }
}
