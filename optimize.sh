#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck > /dev/null && shellcheck "$0"

export PATH=$PATH:/root/.cargo/bin

# This parameter allows us to mount a folder into docker container's "/code"
# and build "/code/contracts/mycontract".
contractdir="$1"
echo "Building contract in $(realpath -m "$contractdir")"

(
  cd "$contractdir"

  # Linker flag "-s" for stripping (https://github.com/rust-lang/cargo/issues/3483#issuecomment-431209957)
  RUSTFLAGS='-C link-arg=-s' cargo wasm --locked
  wasm-opt -Os target/wasm32-unknown-unknown/release/*.wasm -o contract.wasm

  sha256sum contract.wasm > hash.txt
)

echo "done"
