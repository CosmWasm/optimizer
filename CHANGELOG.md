# CHANGELOG

## 0.9.0

- Bump Rust to 1.45.0

## 0.8.0

- Rename Github repo from `confio/cosmwasm-opt` to `CosmWasm/rust-optimizer`
- Rename Docker name from `confio/cosmwasm-opt` to `cosmwasm/rust-optimizer`
- Bump Rust to 1.43.1

## 0.7.3

- Avoid using CosmWasm shortcut `cargo wasm` since the config file
  `.cargo/config` is not included in creates.io source code publications.

## 0.7.2

**Note:** This version cannot be used for reproducible builds from crates.io
sources and should not be used. 0.7.0, 0.7.1 and 0.7.3 unaffected.

- Avoid using `web-pack` by doing the stripping directly. This removes the
  dependency in `wasm-bindgen` in contracts.
- Bump Rust to 1.41.1

## 0.7.1

- Avoid building schema during optimization. This belongs to the development
  flow, not to the reproducible build flow.

## 0.7.0

- Bump emscripten to 1.39.8-fastcomp
- Bump Rust to 1.41.0
