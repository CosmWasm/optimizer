#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck > /dev/null && shellcheck "$0"

export PATH=$PATH:/root/.cargo/bin

# This is designed for the case where the repo contrains multiple contracts,
# all in one workspace. (eg. `cosmwasm-plus`). We want to compile them all and
# optimize all the files (multiple ones). We dump them in a top level dir
# called `artifacts` with each contract named properly (`cw20_base.wasm`)
#
# To compile them all, use: `./multi-optimize.sh`

# build all the contracts from the root
# Linker flag "-s" for stripping (https://github.com/rust-lang/cargo/issues/3483#issuecomment-431209957)
RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown --locked

# take all the build artifacts and optimize them
mkdir -p artifacts
for wasm in target/wasm32-unknown-unknown/release/*.wasm; do
  name=$(basename "$wasm")
  wasm-opt -Os "$wasm" -o "artifacts/$name"
done

(
  cd artifacts
  sha256sum -- *.wasm > contracts.txt
)

echo "done"
