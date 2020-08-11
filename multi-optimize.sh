#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck > /dev/null && shellcheck "$0"

export PATH=$PATH:/root/.cargo/bin

# This is designed for the case where the repo contrains multiple contracts,
# all in one workspace. (eg. `cosmwasm-plus`). We want to compile them all and
# optimize all the files (multiple ones). We dump them in a top level dir
# called `artifacts` with each contract named properly (`cw20_base.wasm`)
#
# To compile them all, use: `./multi-optimize.sh ./contracts/*`

# build all the contracts
for contractdir in "$@"; do
  (
    echo "$contractdir"
    cd "$contractdir"
    # Linker flag "-s" for stripping (https://github.com/rust-lang/cargo/issues/3483#issuecomment-431209957)
    # Note that shortcuts from .cargo/config are not available in source code packages from crates.io
    RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown --locked
  )
done

mkdir -p artifacts

# take all the build artifacts and optimize them
for wasm in target/wasm32-unknown-unknown/release/*.wasm; do
  name=$(basename "$wasm")
  wasm-opt -Os "$wasm" -o "artifacts/$name"
done

(
  cd artifacts
  sha256sum -- *.wasm > hash.txt
)

echo "done"
