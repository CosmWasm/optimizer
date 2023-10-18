#!/bin/ash
# shellcheck shell=dash
# See https://www.shellcheck.net/wiki/SC2187
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

export PATH="$PATH:/root/.cargo/bin"

# Suffix for non-Intel built artifacts
MACHINE=$(uname -m)
SUFFIX=${MACHINE#x86_64}
SUFFIX=${SUFFIX:+-$SUFFIX}

# Debug toolchain and default Rust version
rustup toolchain list
cargo --version

echo "Info: RUSTC_WRAPPER=$RUSTC_WRAPPER"

echo "Info: sccache stats before build"
sccache -s

mkdir -p artifacts
rm -f artifacts/checksums_intermediate.txt

# There are two cases here
# 1. All contracts (or one) are included in the root workspace (eg. `cosmwasm-template`, `cosmwasm-examples`, `cosmwasm-plus`)
#    In this case, we pass no argument, just mount the proper directory.
# 2. Contracts are excluded from the root workspace, but import relative paths from other packages (only `cosmwasm`).
#    In this case, we mount root workspace and pass in a path `docker run <repo> ./contracts/hackatom`

# This parameter allows us to mount a folder into docker container's "/code"
# and build "/code/contracts/mycontract".
# Note: if CONTRACTDIR is "." (default in Docker), this ends up as a noop
for CONTRACTDIR in "$@"; do
  echo "Building contract in $(realpath "$CONTRACTDIR") ..."
  (
    cd "$CONTRACTDIR"
    /usr/local/bin/bob
  )

  # wasm-optimize on all results
  for WASM in /target/wasm32-unknown-unknown/release/*.wasm; do
    NAME=$(basename "$WASM" .wasm)${SUFFIX}.wasm
    echo "Creating intermediate hash for $NAME ..."
    sha256sum -- "$WASM" | tee -a artifacts/checksums_intermediate.txt
    echo "Optimizing $NAME ..."
    # --signext-lowering is needed to support blockchains runnning CosmWasm < 1.3. It can be removed eventually
    wasm-opt -Os --signext-lowering "$WASM" -o "artifacts/$NAME"
  done
done

# create hash
echo "Creating hashes ..."
(
  cd artifacts
  sha256sum -- *.wasm | tee checksums.txt
)

echo "Info: sccache stats after build"
sccache -s

echo "done"
