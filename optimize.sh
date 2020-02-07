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

  contract_name=$(grep --extended-regexp -e 'name\s*=\s*"([-_a-zA-Z0-9]+)"' Cargo.toml | cut -d= -f2 | tr -dc "\-_a-zA-Z0-9")
  echo "Found contract name $contract_name"

  wasm-pack build --release --out-dir "${outdir}" -- --locked
  wasm-opt -Os "${outdir}"/*.wasm -o "$contract_name.wasm"
  sha256sum "$contract_name.wasm" > "$contract_name.sha256"
  cargo run --example schema
)

echo "done"
