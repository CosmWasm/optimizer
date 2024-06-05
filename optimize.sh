#!/bin/ash
# shellcheck shell=dash
# See https://www.shellcheck.net/wiki/SC2187
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

export PATH="$PATH:/root/.cargo/bin"

# Debug toolchain and default Rust version
rustup toolchain list
cargo --version

# Prepare artifacts directory for later use
mkdir -p artifacts

# Delete previously built artifacts. Those can exist if the image is called
# with a cache mounted to /target. In cases where contracts are removed over time,
# old builds in cache should not be contained in the result of the next build.
rm -f /target/wasm32-unknown-unknown/release/*.wasm

# There are two cases here
# 1. The contract is included in the root workspace (eg. `cosmwasm-template`)
#    In this case, we pass no argument, just mount the proper directory.
# 2. Contracts are excluded from the root workspace, but import relative paths from other packages (only `cosmwasm`).
#    In this case, we mount root workspace and pass in a path `docker run <repo> ./contracts/hackatom`

# This parameter allows us to mount a folder into docker container's "/code"
# and build "/code/contracts/mycontract".
# The default value for $1 is "." (see CMD in the Dockerfile).

# Ensure we get exactly one argument and this is a directory (the path to the Cargo project to be built)
if [ "$#" -ne 1 ] || ! [ -d "$1" ]; then
  echo "Usage: $0 DIRECTORY" >&2
  exit 1
fi
PROJECTDIR="$1"
echo "Building project $(realpath "$PROJECTDIR") ..."
(
  cd "$PROJECTDIR"
  /usr/local/bin/bob
)

echo "Optimizing artifacts ..."
for WASM in /target/wasm32-unknown-unknown/release/*.wasm; do
  OUT_FILENAME=$(basename "$WASM")
  echo "Optimizing $OUT_FILENAME ..."
  # --signext-lowering is needed to support blockchains runnning CosmWasm < 1.3. It can be removed eventually
  wasm-opt -Os --signext-lowering "$WASM" -o "artifacts/$OUT_FILENAME"
done

echo "Post-processing artifacts..."
(
  cd artifacts
  # create hashes
  sha256sum -- *.wasm | tee checksums.txt
)

echo "done"
