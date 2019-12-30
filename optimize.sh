#!/bin/sh

set -e

export PATH=$PATH:/root/.cargo/bin
outdir=$(mktemp -d)

# We must previously install wasm-bindgen-cli to allow this to run as non-root user
wasm-pack build -m no-install --release --out-dir "${outdir}"

wasm-opt -Os "${outdir}"/*.wasm -o contract.wasm

sha256sum contract.wasm > hash.txt

cargo run --example schema

echo "done"