# CHANGELOG

## [Unreleased]

## [0.17.0] - 2025-06-26

- Bump Rust to 1.86.0. ([#168])
- Remove `--signext-lowering` flag from `wasm-opt`. ([#168])

Note that contracts built with this version _require CosmWasm 3.0+_ on the chain and cannot be
uploaded to chains running lower versions.

[#168]: https://github.com/CosmWasm/optimizer/pull/168

## [0.16.1] - 2024-10-11

- Bump Rust to current stable 1.81.0

## [0.16.0] - 2024-06-06

- Bump Rust to current stable 1.78.0.
- Remove "-aarch64" suffix from filename when .wasm files are built on an ARM system.
  There is no good reason for those given that the builder images for ARM have a
  different name and each builder image produces different results. ([#151])
- Allow configuring multiple builds in a codebase using `[package.metadata.optimizer]` settings
  in `Cargo.toml` ([#148], [#156]). E.g.
  ```
  [package.metadata.optimizer]
  standard-build = true
  builds = [
    { name = "debug", features = ["debug"] }
    { name = "tokenfactory", features = ["tokenfactory"], default-features = false },
  ]
  ```

[#151]: https://github.com/CosmWasm/optimizer/issues/151
[#148]: https://github.com/CosmWasm/optimizer/pull/148
[#156]: https://github.com/CosmWasm/optimizer/pull/156

## [0.15.1] - 2024-02-25

- Bump Rust to current stable 1.75.0.
- Bump wasm-opt to version v116.

## [0.15.0] - 2023-11-13

- **cosmwasm/rust-optimizer and cosmwasm/workspace-optimizer were merged into cosmwasm/optimizer.**
  The old image names are preserved but deprecated.
- **cosmwasm/rust-optimizer-arm64 and cosmwasm/workspace-optimizer-arm64 were merged into cosmwasm/optimizer-arm64.**
  The old image names are preserved but deprecated.
- Bump Rust to current stable 1.73.0.
- Use builder tool `bob` for both single contract and workspace builds ([#134])
- Remove sccache.
  This caching was only useful when compiling multiple independent Rust projects. cosmwasm/rust-optimizer currently supports that, but this feature is not needed anymore and should be removed. Instead, users can call rust-optimizer once for each contract or use workspace-optimizer.
- Both rust-optimizer and workspace-optimizer now take exactly one path argument for
  the project to be built. The Docker images default to `.` if not set.
  This means calling rust-optimizer for multiple projects at once is now unsupported.
  You should either migrate to a workspace or change the call from:

  ```sh
  docker run --rm -v "$(pwd)":/code \
    --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    cosmwasm/rust-optimizer:0.14.0 ./contracts/*/
  ```

  to

  ```sh
  for contract_dir in contracts/*; do
    docker run --rm -v "$(pwd)":/code \
      --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
      --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
      cosmwasm/rust-optimizer:0.15.0 "$contract_dir"
  done
  ```

## [0.14.0] - 2023-07-28

- Update to binaryen v114.
- Bump Rust to current stable v1.71.0.
- Add the `--signext-lowering` flag to `wasm-opt`. ([#127])

## [0.13.0] - 2023-06-20

### Changed

- Moved target folder from `/code/target` to `/target`. To upgrade your caller code use one of those diffs.
  For rust-optimizer:

  ```diff
   docker run --rm -v "$(pwd)":/code \
  -  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  +  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
     --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  -  cosmwasm/rust-optimizer:0.12.13
  +  cosmwasm/rust-optimizer:0.13.0
  ```

  or for workspace-optimizer:

  ```diff
   docker run --rm -v "$(pwd)":/code \
  -  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  +  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
     --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  -  cosmwasm/workspace-optimizer:0.12.11
  +  cosmwasm/workspace-optimizer:0.13.0
  ```

- Bump Rust to 1.69.0

## [0.12.13] - 2023-03-30

- Bump Rust to 1.68.2.\
  This contains [cargo's sparse protocol support](https://blog.rust-lang.org/2023/03/09/Rust-1.68.0.html) which is enabled here.

## [0.12.12] - 2023-03-06

- Bump Rust to 1.67.1

## [0.12.11] - 2022-12-22

- Bump Rust to 1.66.0

## [0.12.10] - 2022-11-18

- Update to binaryen v110.
- Bump Rust to current stable v1.65.0

## 0.12.9

- Bump Rust to current stable v1.64.0

## 0.12.8

- Add --lib to cargo build

## 0.12.7

- Support incremental workspace optimizations ([#84])
- Bump Rust to current stable v1.63.0.

## 0.12.6

- Bump Rust to current stable v1.60.0.

## 0.12.5

- Update to binaryen v105.
- Bump Rust to current stable v1.58.1.

## 0.12.4

- Update to binaryen v102. Build binaryen from sources and test it in both,
  Intel and Arm 64.
- Adds the "-arm64" suffix to Arm 64 image names. Also adds a suffix to non-x86_64 (Intel 64 bits) built artifacts.
- Add missing "is directory" check for build workspace, to avoid panic on extra typescript files.

## 0.12.3

- Port workspace-optimizer Python script to Rust to reduce image sizes.
- Bump Rust to 1.55.0

## 0.12.2

- Support for arm64 Docker images (not published; see ([#60])).

[#60]: https://github.com/CosmWasm/rust-optimizer/issues/60

## 0.12.1

- Use the Docker builder pattern to reduce image sizes by preventing temporary files from
  entering the `cosmwasm/rust-optimizer` and `cosmwasm/workspace-optimizer` images

## 0.12.0

- Reorganize project to use multi-stage builds instead of different docker files. This
  way no `cosmwasm/base-optimizer` image is expected on DockerHub.

## 0.11.5

- Bump Rust to 1.54.0

## 0.11.4

- Bump Rust to 1.53.0

## 0.11.3

- Remove `-n` from `echo` to flush logs early.
- Consolidate log style.
- Revert shell to `/bin/ash`.

## 0.11.2

- Fix target path for \*.wasm files.

## 0.11.1

- Issues when running `workspace-optimizer` in CircleCI (cosmwasm-plus #273).
  Revert to using `/bin/sh` for shell.

## 0.11.0

- Use precompiled sccache
- Reduce image size by deleting unnecessary files
- `cosmwasm/workspace-optimizer`: migrate from Rust nightly to stable (1.51.0)
- Migrate to alpine-based Rust images for smaller images

## 0.10.9

- `cosmwasm/rust-optimizer`: bump Rust to 1.51.0

## 0.10.8

- `cosmwasm/rust-optimizer`: bump Rust to 1.50.0
- `cosmwasm/workspace-optimizer`: bump Rust to nightly-2021-03-01

## 0.10.7

- Add shared build cache (sccache) to rust-optimizer

## 0.10.6

- Add support for building multiple non-workspace contracts at once (#25)

## 0.10.5

- Remove `trzeci/emscripten` dependency and install `wasm-opt` manually
- Upgrade `wasm-opt` to version 96

## 0.10.4

- `cosmwasm/rust-optimizer`: bump Rust to 1.47.0
- `cosmwasm/workspace-optimizer`: bump Rust to nightly-2020-10-14

## 0.10.3

- `cosmwasm/rust-optimizer`: bump Rust to 1.45.2

## 0.9.1

- Bump Rust to 1.45.2

## 0.10.2

- Split into `cosmwasm/rust-optimizer` and `cosmwasm/workspace-optimizer` to
  restore 0.9 stability for single contract builds while building adding support
  for monorepo builds. `cosmwasm/workspace-optimizer` now uses Rust
  `nightly-2020-08-20`.

## 0.10.1

- Rename `./artifacts/contracts.txt` to `./artifacts/checksums.txt`.

## 0.10.0

- Initial attempt to support workspace repos. Contracts are now writtien to
  `./articfacts/{contract name}.wasm` and `./artifacts/contracts.txt`.

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

[unreleased]: https://github.com/CosmWasm/rust-optimizer/compare/v0.17.0...HEAD
[0.17.0]: https://github.com/CosmWasm/rust-optimizer/compare/v0.16.1...v0.17.0
[0.16.1]: https://github.com/CosmWasm/rust-optimizer/compare/v0.16.0...v0.16.1
[0.16.0]: https://github.com/CosmWasm/rust-optimizer/compare/v0.15.1...v0.16.0
[0.15.1]: https://github.com/CosmWasm/rust-optimizer/compare/v0.15.0...v0.15.1
[0.15.0]: https://github.com/CosmWasm/rust-optimizer/compare/v0.14.0...v0.15.0
[0.14.0]: https://github.com/CosmWasm/rust-optimizer/compare/v0.13.0...v0.14.0
[0.13.0]: https://github.com/CosmWasm/rust-optimizer/compare/v0.12.13...v0.13.0
[0.12.13]: https://github.com/CosmWasm/rust-optimizer/compare/v0.12.12...v0.12.13
[0.12.12]: https://github.com/CosmWasm/rust-optimizer/compare/v0.12.11...v0.12.12
[0.12.11]: https://github.com/CosmWasm/rust-optimizer/compare/v0.12.10...v0.12.11
[0.12.10]: https://github.com/CosmWasm/rust-optimizer/compare/v0.12.9...v0.12.10
