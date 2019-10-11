#!/bin/sh

set -e

export PATH=$PATH:/root/.cargo/bin
outdir=$(mktemp -d)

# We must previously install wasm-bindgen-cli to allow this to run as non-root user
echo wasm-pack build -m no-install --release --out-dir "${outdir}"
wasm-pack build --out-dir "${outdir}"

echo wasm-opt -Os "${outdir}"/*.wasm -o contract.wasm
wasm-opt -Os "${outdir}"/*.wasm -o contract.wasm

echo "done"