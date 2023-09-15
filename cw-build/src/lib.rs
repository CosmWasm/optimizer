pub mod contract;
pub mod workspace;

use serde::Deserialize;
use std::{fs, path::PathBuf};

pub const OUTPUT_DIR: &str = "/target/wasm32-unknown-unknown/release";
pub const CARGO_PATH: &str = "cargo";

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

/// Get all the builds and wasm name from the `Cargo.toml` file at the given path.
pub fn parse_toml_at(path: &PathBuf) -> (String, Option<Vec<Build>>) {
    let contract_cargo_toml = fs::read_to_string(path.join("Cargo.toml")).unwrap();
    let PackageCargoToml { package } = toml::from_str(&contract_cargo_toml).unwrap();

    let wasm_name = package.name.replace("-", "_");

    let builds = package
        .metadata
        .and_then(|metadata| metadata.optimizer)
        .and_then(|optimizer| optimizer.builds);
    (wasm_name, builds)
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

/// Returns the default wasm path name formatted as `<output_dir>/<wasm_name>.wasm`
fn default_wasm_path(wasm_name: &str) -> String {
    format!("{}/{}.wasm", OUTPUT_DIR, wasm_name)
}
/// Returns path name formatted as `<output_dir>/<wasm_name>-<build_name>.wasm`
fn wasm_path(wasm_name: &str, build_name: &str) -> String {
    if build_name.is_empty() {
        default_wasm_path(wasm_name)
    } else {
        format!("{}/{}-{}.wasm", OUTPUT_DIR, wasm_name, build_name)
    }
}
