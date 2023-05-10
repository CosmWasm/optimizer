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
pub struct WorkspaceCargoToml {
    workspace: Workspace,
}

#[derive(Deserialize, Debug)]
pub struct Workspace {
    members: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct PackageCargoToml {
    package: Package,
}

#[derive(Deserialize, Debug)]
pub struct Package {
    name: String,
    metadata: Option<OptimizerMetadata>,
}

#[derive(Deserialize, Debug)]
pub struct OptimizerMetadata {
    optimizer: Option<Optimizer>,
}

#[derive(Deserialize, Debug)]
pub struct Optimizer {
    features: Option<Vec<String>>,
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

const OUTPUT_DIR: &str = "target/wasm32-unknown-unknown/release";

/// Build the contract at the path *contract* with the provided *features*.
fn build_contract(contract: &PathBuf, wasm_name: &String, feature: Option<&String>) {
    println!("Building {:?} with features {:?}", contract, feature);

    let mut args = vec![
        "build",
        "--release",
        "--lib",
        "--target=wasm32-unknown-unknown",
        "--locked",
    ].into_iter().map(|arg| arg.to_string()).collect::<Vec<String>>();

    if let Some(feature) = feature {
        args.push(format!("--features={}", feature));
    }

    let mut child = Command::new(CARGO_PATH)
        .args(&args)
        .env("RUSTFLAGS", "-C link-arg=-s")
        .current_dir(canonicalize(contract).unwrap())
        .spawn()
        .unwrap();
    let error_code = child.wait().unwrap();
    assert!(error_code.success());

    // The feature wasm should be suffixed with the feature name to differentiate itself
    if let Some(feature) = feature {
        let input_wasm_path = format!("{}/{}.wasm", OUTPUT_DIR, wasm_name);
        let output_wasm_path = format!("{}/{}-{}.wasm", OUTPUT_DIR, wasm_name, feature);
        fs::rename(&input_wasm_path, &output_wasm_path).expect("Failed to rename the output file");
    }
}

fn main() {
    let file = fs::read_to_string("Cargo.toml").unwrap();
    let cargo_toml: WorkspaceCargoToml = toml::from_str(&file).unwrap();
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
        let contract_cargo_toml = fs::read_to_string(contract.join("Cargo.toml")).unwrap();
        let PackageCargoToml {
            package,
        } = toml::from_str(&contract_cargo_toml).unwrap();

        let wasm_name = package.name.replace("-", "_");

        let features = package.metadata
            .and_then(|metadata| metadata.optimizer)
            .and_then(|optimizer| optimizer.features);
        // build contract for each feature
        if let Some(features) = features {
            for feature in features.iter() {
                build_contract(contract, &wasm_name, Some(feature));
            }
        }
        // build contract without features
        build_contract(contract, &wasm_name, None)
    }
}
