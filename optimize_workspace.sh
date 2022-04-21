#!/bin/ash
# shellcheck shell=dash
# See https://www.shellcheck.net/wiki/SC2187
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

[ -n "${GIT_CREDENTIALS:=""}" ] && echo "$GIT_CREDENTIALS" >~/.git-credentials && git config --global credential.helper store
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
echo "Building artifacts in workspace ..."
/usr/local/bin/build_workspace

mkdir -p artifacts
echo "Creating intermediate hashes ..."
sha256sum -- target/wasm32-unknown-unknown/release/*.wasm | tee artifacts/checksums_intermediate.txt

echo "Optimizing artifacts in workspace ..."
TMPARTIFACTS=$(mktemp -p "$(pwd)" -d artifacts.XXXXXX)
# Optimize artifacts
(
  cd "$TMPARTIFACTS"

  for WASM in ../target/wasm32-unknown-unknown/release/*.wasm; do
    NAME=$(basename "$WASM" .wasm)${SUFFIX}.wasm
    echo "Optimizing $NAME ..."
    wasm-opt -Os "$WASM" -o "$NAME"
  done
  echo "Moving wasm files ..."
  mv ./*.wasm ../artifacts
)
rm -rf "$TMPARTIFACTS"
echo "Post-processing artifacts in workspace ..."
(
  cd artifacts
  sha256sum -- *.wasm | tee checksums.txt
)

echo "done"
