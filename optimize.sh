#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck > /dev/null && shellcheck "$0"

export PATH=$PATH:/root/.cargo/bin
outdir=$(mktemp -d)
# This parameter allows us to mount a folder into docker container's "/code"
# and build "/code/contracts/mycontract".
contractdir="$1"
echo "Building contract in $(realpath -m "$contractdir")"

(
  cd "$contractdir"
  wasm-pack build --release --out-dir "${outdir}" -- --locked
  wasm-opt -Os "${outdir}"/*.wasm -o contract.wasm
  sha256sum contract.wasm > hash.txt

  # Is this necessary here? This step takes a long time because it compiles all devDependencies, including the whole VM.
  echo "Rebuilding schema ..."
  cargo run --example schema
)

echo "done"
