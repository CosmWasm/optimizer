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

rustup toolchain list
cargo --version

# Delete already built artifacts
rm -f target/wasm32-unknown-unknown/release/*.wasm

# Build artifacts
echo "Building artifacts in workspace..."
/usr/local/bin/build_workspace

mkdir -p artifacts
echo "Optimizing artifacts in workspace..."
TMPARTIFACTS=$(mktemp -p "$(pwd)" -d artifacts.XXXXXX)
# Optimize artifacts
(
  cd "$TMPARTIFACTS"

  for WASM in ../target/wasm32-unknown-unknown/release/*.wasm; do
    BASENAME=$(basename "$WASM" .wasm)
    NAME=${BASENAME}${SUFFIX}
    FILENAME=${NAME}.wasm

    SHA256=$(sha256sum -- "$WASM" | sed 's/..\/target/target/g')
    INTERMEDIATES="../artifacts/checksums_intermediate.txt"
    if grep -Fxq "$SHA256" "$INTERMEDIATES"; then
      echo "$BASENAME unchanged. Skipping optimization."
    else
      grep -vs "$BASENAME" "$INTERMEDIATES" >tmp_shas && mv -f tmp_shas "$INTERMEDIATES"
      echo "Creating intermediate hash for ${BASENAME}..."
      echo "$SHA256" | tee -a "$INTERMEDIATES" >/dev/null
      echo "Optimizing ${BASENAME}..."
      wasm-opt -Os "$WASM" -o "$FILENAME"
      echo "Moving wasm files..."
      mv ./*.wasm ../artifacts
    fi
  done
)
rm -rf "$TMPARTIFACTS"
echo "Post-processing artifacts in workspace..."
(
  cd artifacts
  sha256sum -- *.wasm | tee checksums.txt
)

echo "done"
