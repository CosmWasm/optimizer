#!/usr/bin/env python3

# Build script for cargo workspaces

CARGO_PATH="cargo"
PACKAGE_PREFIX="contracts/"

import glob
import os
import shutil
import subprocess
import toml

def log(*args):
    print(*args, flush=True)

with open("Cargo.toml") as file:
    document = toml.load(file)
    members = document['workspace']['members']

log("Found workspace member entries:", members)

all_packages = []
for member in members:
    all_packages.extend(glob.glob(member))
all_packages.sort()
log("Package directories:", all_packages)

contract_packages = [p for p in all_packages if p.startswith(PACKAGE_PREFIX)]
log("Contracts to be built:", contract_packages)

artifacts_dir = os.path.realpath("artifacts")
os.makedirs(artifacts_dir, exist_ok=True)

for contract in contract_packages:
    log("Building {} ...".format(contract))
    # Rust nightly and unstable-options is needed to use --out-dir
    cmd = [CARGO_PATH, "-Z=unstable-options", "build", "--release", "--target=wasm32-unknown-unknown", "--locked", "--out-dir=./contract_artifacts"]
    os.environ["RUSTFLAGS"] = "-C link-arg=-s"
    subprocess.check_call(cmd, cwd=contract)
    for wasm in glob.glob(os.path.realpath(contract) + "/contract_artifacts/*wasm"):
        log("Successfully built", wasm)
        shutil.copy(wasm, artifacts_dir)
