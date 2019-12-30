#!/bin/sh

set -e

export PATH=$PATH:/root/.cargo/bin
outdir=$(mktemp -d)

wasm-pack build --release --out-dir "${outdir}"

wasm-opt -Os "${outdir}"/*.wasm -o contract.wasm

sha256sum contract.wasm > hash.txt

cargo run --example schema

echo "done"