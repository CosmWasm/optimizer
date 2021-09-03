use glob::glob;
use serde::Deserialize;
use std::{
    fs,
    io::{self, Write},
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

pub fn parse_cargo_toml(path: String) -> CargoToml {
    let file = fs::read_to_string(path).expect("Something went wrong reading the file");

    let cargo_toml: CargoToml = toml::from_str(&file).unwrap();
    cargo_toml
}

fn main() {
    let cargo_toml = parse_cargo_toml("Cargo.toml".to_string());
    let members = cargo_toml.workspace.members;
    println!("Found workspace member entries: {:?}", members);

    let mut all_packages = vec![];

    for member in members.into_iter() {
        let g = glob(&member).expect("bar");
        for entry in g {
            all_packages.push(entry.unwrap());
        }
    }

    all_packages.sort();

    println!("Package directories: {:?}", all_packages);

    let contract_packages = all_packages
        .into_iter()
        .filter(|p| p.starts_with(PACKAGE_PREFIX))
        .collect::<Vec<_>>();

    println!("Contracts to be built: {:?}", contract_packages);

    for contract in contract_packages {
        println!("Building {:?} ...", contract);

        let output = Command::new(CARGO_PATH)
            .args(&[
                "build",
                "--release",
                "--target=wasm32-unknown-unknown",
                "--locked",
            ])
            .env("RUSTFLAGS", "-C link-arg=-s")
            .output()
            .expect("baz");

        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();

        assert!(output.status.success());
    }
}
