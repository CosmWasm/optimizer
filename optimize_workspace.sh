#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

export PATH="$PATH:/root/.cargo/bin"

rustup toolchain list
cargo --version

# Build artifacts
echo -n "Building artifacts in workspace..."
/usr/local/bin/build_workspace.py
echo "done."

echo -n "Optimizing artifacts in workspace..."
# Start clean
rm -rf ./artifacts
mkdir artifacts
# Optimize and postprocess artifacts
(
  cd artifacts

  for WASM in ../target/wasm32-unknown-unknown/release/*/*.wasm
  do
    echo -n "Optimizing $WASM..."
    BASE=$(basename "$WASM")
    wasm-opt -Os -o "$BASE" "$WASM"
    chmod -x "$BASE"
    echo "done."
  done
  sha256sum -- *.wasm >checksums.txt
)
echo "done."
