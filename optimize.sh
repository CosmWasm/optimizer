#!/usr/bin/env bash

# http://redsymbol.net/articles/unofficial-bash-strict-mode/
set -euo pipefail
IFS=$'\n\t'

export PATH="$PATH:/root/.cargo/bin"
mkdir -p artifacts

RUSTFLAGS="-C link-arg=-s" cargo build --release --lib --target=wasm32-unknown-unknown
for i in $(cargo metadata --no-deps --format-version 1 | jq -r '.workspace_members[]'); do
        binary="$(echo "$i" | awk '{print $1}' | tr '-' '_').wasm"
        path="$(echo "$i" | awk '{print $3}' | sed -nr 's/\(path\+file\:\/\/(.+)\)/\1/p')"
        path="${path#$(pwd)/}"
        if [[ $path == contracts* ]]; then
                echo "Optimizing $binary…"
                wasm-opt -Os --signext-lowering                         \
			"target/wasm32-unknown-unknown/release/$binary" \
			-o "artifacts/$binary"
        fi
done

(
  cd artifacts
  sha256sum -- *.wasm | tee checksums.txt
)
