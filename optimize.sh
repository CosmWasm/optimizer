#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck > /dev/null && shellcheck "$0"

export PATH=$PATH:/root/.cargo/bin
outdir=$(mktemp -d)

wasm-pack build --release --out-dir "${outdir}" -- --locked

wasm-opt -Os "${outdir}"/*.wasm -o contract.wasm

sha256sum contract.wasm > hash.txt

cargo run --example schema

echo "done"
