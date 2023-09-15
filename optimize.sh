#!/usr/bin/env bash

echo "Info: optimize.sh"

set -o errexit -o nounset -o pipefail
trap 'echo >&2 "Error on line $LINENO"' ERR
command -v shellcheck >/dev/null && shellcheck "$0"

export PATH=$PATH:/root/.cargo/bin

# Suffix for non-Intel built artifacts
MACHINE=$(uname -m)
SUFFIX=${MACHINE#x86_64}
SUFFIX=${SUFFIX:+-$SUFFIX}

echo "Info: RUSTC_WRAPPER=$RUSTC_WRAPPER"

echo "Info: sccache stats before build"
sccache -s

mkdir -p artifacts
rm -f artifacts/checksums_intermediate.txt

for CONTRACTDIR in "$@"; do
  echo "Building contract in $(realpath "$CONTRACTDIR") ..."
    if [ ! -f "$CONTRACTDIR/Cargo.toml" ]; then
      echo "Cargo.toml not found in $CONTRACTDIR. Skipping this directory."
      continue
    fi
  (
    cd "$CONTRACTDIR"
    echo "Info: Building in $CONTRACTDIR"

    /usr/local/bin/cw-build
  )
  
  echo "Info: Finished building in $CONTRACTDIR"

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

# Create hash
echo "Creating hashes ..."
(
  cd artifacts
  sha256sum -- *.wasm | tee checksums.txt
)

echo "Info: sccache stats after build"
sccache -s

echo "done"
