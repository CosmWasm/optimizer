#!/usr/bin/env python3

# Build script for cargo workspaces

CARGO_PATH="cargo"
PACKAGE_PREFIX="contracts/"

import glob
import os
import shutil
import stat
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

for contract in contract_packages:
    log("Building {} ...".format(contract))

    cmd = [CARGO_PATH, "build", "--release", "--target=wasm32-unknown-unknown", "--locked"]
    os.environ["RUSTFLAGS"] = "-C link-arg=-s"
    subprocess.check_call(cmd, cwd=contract)
