use std::{fs, path::Path, process::Command};

use serde::Deserialize;

use crate::cargo_toml::package::{Build, BuildSettings};

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct ParsedPackage {
    pub name: String,
    pub standard_build: bool,
    /// Builds that are created on top of the standard build
    pub builds: Vec<Build>,
}

impl ParsedPackage {
    /// Build a contract with all the requested builds defined in `[package.metadata.optimizer]`
    pub fn build(self, path: &Path) {
        let ParsedPackage {
            name: package_name,
            standard_build,
            mut builds,
        } = self;
        if standard_build {
            builds.push(Build::default()); // add contract with default features
        }

        // Build all the requested builds
        for build in builds.into_iter() {
            build.build(path, &package_name);
        }
    }
}

impl Build {
    /// Build the contract at the path *contract*.
    pub fn build(self, contract: &Path, package_name: &str) {
        let Build {
            name: build_name,
            settings:
                BuildSettings {
                    features,
                    default_features,
                },
        } = self;

        let features = features.unwrap_or_default();

        eprintln!("Building {} with features {:?}", package_name, features);

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

        // Add features to command
        let features_arg = features.into_iter().collect::<Vec<String>>().join(", ");
        args.push(format!("--features={}", features_arg));

        // add default features to command
        if let Some(false) = default_features {
            args.push("--no-default-features".to_string());
        }

        // Run the build
        let mut child = Command::new(crate::CARGO_PATH)
            .args(&args)
            .env("RUSTFLAGS", "-C link-arg=-s")
            .current_dir(fs::canonicalize(contract).unwrap())
            .spawn()
            .unwrap();
        let error_code = child.wait().unwrap();
        assert!(error_code.success());

        // Rename to name formatted as `<output_dir>/<wasm_name>-<build_name>.wasm`
        if !build_name.is_empty() {
            let input_wasm_path = default_wasm_path(package_name);
            let output_wasm_path = wasm_path(package_name, &build_name);
            fs::rename(&input_wasm_path, &output_wasm_path)
                .expect("Failed to rename the output file");
        }
    }
}

pub const OUTPUT_DIR: &str = "/target/wasm32-unknown-unknown/release";

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
