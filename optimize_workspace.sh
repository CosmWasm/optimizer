#!/bin/ash
# shellcheck shell=dash
# See https://www.shellcheck.net/wiki/SC2187
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

export PATH="$PATH:/root/.cargo/bin"

rustup toolchain list
cargo --version

# Delete already built artifacts
rm -f target/wasm32-unknown-unknown/release/*.wasm

# Build artifacts
echo "Building artifacts in workspace ..."
/usr/local/bin/build_workspace.py

echo "Optimizing artifacts in workspace ..."
mkdir -p artifacts
TMPARTIFACTS=$(mktemp -p "$(pwd)" -d artifacts.XXXXXX)
# Optimize artifacts
(
  cd "$TMPARTIFACTS"

  for WASM in ../target/wasm32-unknown-unknown/release/*.wasm; do
    echo "Optimizing $WASM ..."
    BASE=$(basename "$WASM")
    wasm-opt -Os -o "$BASE" "$WASM"
    chmod -x "$BASE"
  done
  mv ./*.wasm ../artifacts
)
rm -rf "$TMPARTIFACTS"
echo "Post-processing artifacts in workspace ..."
(
  cd artifacts
  sha256sum -- *.wasm >checksums.txt
)
