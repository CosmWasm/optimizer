use std::{
    collections::HashMap,
    fs::{self, canonicalize},
    path::PathBuf,
    process::Command,
};

use crate::{default_wasm_path, wasm_path, Build, BuildName, Feature};

/// Build the contract at the path *contract_dir*.
pub fn build_contract(contract_dir: &PathBuf) {
    // read the toml file
    // get the wasm name and builds from the Cargo.toml
    let (wasm_name, builds) = crate::parse_toml_at(contract_dir);

    // Keep track of the unique builds to prevent re-compiling the same contract
    // K: features V: build name of build with those features
    let mut built: HashMap<Vec<Feature>, BuildName> = HashMap::new();

    // Build all the requested builds
    if let Some(builds) = builds {
        for build in builds.into_iter() {
            // Sort features so feature ordering doesn't matter.
            let mut features = build.features.clone().unwrap_or_default();
            features.sort();

            if built.contains_key(&features) {
                // build already exists, copy the wasm file with identical features to a new build name
                let built_wasm_name = built.get(&features).unwrap();
                fs::copy(
                    crate::wasm_path(&wasm_name, built_wasm_name),
                    crate::wasm_path(&wasm_name, &build.name),
                )
                .expect("Failed to copy the output file");
                continue;
            }
            perform_build(contract_dir, &wasm_name, &build);
            built.insert(features, build.name.clone());
        }
    }

    if !built.contains_key(&vec![]) {
        // build contract without features or appended name
        perform_build(contract_dir, &wasm_name, &Build::default())
    }
}

/// Build the contract at the path *contract* with the provided *features*.
pub fn perform_build(contract: &PathBuf, wasm_name: &str, build: &Build) {
    let Build {
        name: build_name,
        features,
    } = build;
    let mut features = features.clone().unwrap_or_default();

    eprintln!("Building {} with features {:?}", wasm_name, features);

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
        features.iter().fold(first_feature, |acc, val| {
            format!("{acc}{}", format!(",{}", val))
        })
    } else {
        "".to_string()
    };

    // Add features to command
    args.push(format!("--features={}", featured_arg));

    // Run the build
    let mut child = Command::new(crate::CARGO_PATH)
        .args(&args)
        .env("RUSTFLAGS", "-C link-arg=-s")
        .current_dir(canonicalize(contract).unwrap())
        .spawn()
        .unwrap();
    let error_code = child.wait().unwrap();
    assert!(error_code.success());

    // Rename to name formatted as `<output_dir>/<wasm_name>-<build_name>.wasm`
    if !build_name.is_empty() {
        let input_wasm_path = default_wasm_path(wasm_name);
        let output_wasm_path = wasm_path(wasm_name, build_name);
        fs::rename(&input_wasm_path, &output_wasm_path).expect("Failed to rename the output file");
    }
}
