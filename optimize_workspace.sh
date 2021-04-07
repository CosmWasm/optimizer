#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

export PATH="$PATH:/root/.cargo/bin"

rustup toolchain list
cargo --version

# Delete already built artifacts
rm -f target/wasm32-unknown-unknown/release/*/*.wasm

# Build artifacts
echo -n "Building artifacts in workspace..."
/usr/local/bin/build_workspace.py
echo "done."

echo -n "Optimizing artifacts in workspace..."
mkdir -p artifacts
TMPDIR=$(mktemp -d artifacts.XXX)
# Optimize artifacts
(
  cd "$TMPDIR"

  for WASM in ../target/wasm32-unknown-unknown/release/*/*.wasm
  do
    echo -n "Optimizing $WASM..."
    BASE=$(basename "$WASM")
    wasm-opt -Os -o "$BASE" "$WASM"
    chmod -x "$BASE"
    echo "done."
  done
	mv ./*.wasm ../artifacts
)
rm -rf "$TMPDIR"
echo "done."
echo -n "Post-processing artifacts in workspace..."
(
  cd artifacts
  sha256sum -- *.wasm >checksums.txt
)
echo "done."
