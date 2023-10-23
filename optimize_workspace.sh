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

# Prepare artifacts directory for later use
mkdir -p artifacts
rm -f artifacts/checksums_intermediate.txt

# Delete already built artifacts
rm -f target/wasm32-unknown-unknown/release/*.wasm

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

echo "Optimizing artifacts in workspace..."
TMPARTIFACTS=$(mktemp -p "$(pwd)" -d artifacts.XXXXXX)
# Optimize artifacts
(
  cd "$TMPARTIFACTS"
  INTERMEDIATE_SHAS="../artifacts/checksums_intermediate.txt"

  for WASM in /target/wasm32-unknown-unknown/release/*.wasm; do
    BASENAME=$(basename "$WASM" .wasm)
    NAME=${BASENAME}${SUFFIX}
    OPTIMIZED_WASM=${NAME}.wasm

    INTERMEDIATE_SHA=$(sha256sum -- "$WASM" | sed 's,/target,target,g')

    if test -f "$INTERMEDIATE_SHAS"; then
      echo "Updating intermediate hash for ${BASENAME}..."
      sed -ni "/$BASENAME/!p" "$INTERMEDIATE_SHAS"
    else
      echo "Creating intermediate hash for ${BASENAME}..."
    fi
    echo "$INTERMEDIATE_SHA" >>"$INTERMEDIATE_SHAS"

    echo "Optimizing ${BASENAME}..."
    # --signext-lowering is needed to support blockchains runnning CosmWasm < 1.3. It can be removed eventually
    wasm-opt -Os --signext-lowering "$WASM" -o "$OPTIMIZED_WASM"
    echo "Moving ${OPTIMIZED_WASM}..."
    mv "$OPTIMIZED_WASM" ../artifacts
  done
)
rm -rf "$TMPARTIFACTS"
echo "Post-processing artifacts in workspace..."
(
  cd artifacts
  sha256sum -- *.wasm | tee checksums.txt
)

echo "done"
