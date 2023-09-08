use glob::glob;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::{self, canonicalize},
    path::PathBuf,
    process::Command,
};

const CARGO_PATH: &str = "cargo";
const PACKAGE_PREFIX: &str = "contracts/";

type BuildName = String;
type Feature = String;

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
    builds: Option<Vec<Build>>,
}

/// A build entry that specifies the build of a contract with optional features.
#[derive(Deserialize, Debug, Default)]
pub struct Build {
    /// Name appended to the build output file name.
    pub name: BuildName,
    /// Features to be enabled for this build.
    pub features: Option<Vec<Feature>>,
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

const OUTPUT_DIR: &str = "/target/wasm32-unknown-unknown/release";

/// Build the contract at the path *contract* with the provided *features*.
fn build_contract(contract: &PathBuf, wasm_name: &str, build: &Build) {
    let Build {
        name: build_name,
        features,
    } = build;
    let mut features = features.clone().unwrap_or_default();

    eprintln!("Building {:?} with features {:?}", contract, features);

    let mut args = vec![
        "build",
        "--release",
        "--lib",
        "--target-dir=/target",
        "--target=wasm32-unknown-unknown",
        "--locked",
    ]
    .into_iter()
    .map(|arg| arg.to_string())
    .collect::<Vec<String>>();

    let featured_arg = if !features.is_empty() {
        let first_feature = features.swap_remove(0);
        // construct feature-args
        // e.g. "feature1","feature2","feature3"
        features
            .iter()
            .fold(format!("\"{}\"", first_feature), |acc, val| {
                format!("{acc}{}", format!(",\"{}\"", val))
            })
    } else {
        "".to_string()
    };

    eprintln!("featured_arg: {}", featured_arg);
    // Add features to command
    args.push(format!("--features={}", featured_arg));

    // Run the build
    let mut child = Command::new(CARGO_PATH)
        .args(&args)
        .env("RUSTFLAGS", "-C link-arg=-s")
        .current_dir(canonicalize(contract).unwrap())
        .spawn()
        .unwrap();
    let error_code = child.wait().unwrap();
    assert!(error_code.success());

    // Copy to path name formatted as `<output_dir>/<wasm_name>-<build_name>.wasm`
    // or `<output_dir>/<wasm_name>.wasm` if build_name is empty ("").
    if !build_name.is_empty() {
        let input_wasm_path = format!("{}/{}.wasm", OUTPUT_DIR, wasm_name);
        let output_wasm_path = format!("{}/{}-{}.wasm", OUTPUT_DIR, wasm_name, build_name);
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

    // Keep track of the unique builds to prevent re-compiling the same contract
    // K: features V: build name of build with those features
    let mut built: HashMap<Vec<Feature>, BuildName> = HashMap::new();

    for contract in contract_packages {
        let contract_cargo_toml = fs::read_to_string(contract.join("Cargo.toml")).unwrap();
        let PackageCargoToml { package } = toml::from_str(&contract_cargo_toml).unwrap();

        let wasm_name = package.name.replace("-", "_");

        let builds = package
            .metadata
            .and_then(|metadata| metadata.optimizer)
            .and_then(|optimizer| optimizer.builds);

        // Build all the requested builds
        if let Some(builds) = builds {
            for build in builds.into_iter() {
                // Sort features so feature ordering doesn't matter.
                let mut features = build.features.clone().unwrap_or_default();
                features.sort();

                if built.contains_key(&features) {
                    // build already exists, copy the wasm file with identical features to a new build name
                    continue;
                }
                build_contract(contract, &wasm_name, &build);
                built.insert(features, build.name.clone());
            }
        }

        if !built.contains_key(&vec![]) {
            // build contract without features or appended name
            build_contract(contract, &wasm_name, &Build::default())
        }
    }
}
