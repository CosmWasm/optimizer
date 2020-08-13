# Cosmwasm Optimizing Compiler

[![DockerHub](https://img.shields.io/docker/pulls/cosmwasm/rust-optimizer?style=plastic)](https://hub.docker.com/r/cosmwasm/rust-optimizer)

This is a Docker build with a locked set of dependencies to produce
reproducible builds of cosmwasm smart contracts. It also does heavy
optimization on the build size, using binary stripping and `wasm-opt`.

## Usage

*This works for most cases, for cosmwasm builds see advanced*

The easiest way is to simply use the [published docker image](https://hub.docker.com/r/cosmwasm/rust-optimizer).
You must set the local path to the smart contract you wish to compile and
it will produce an `artifacts` directory with `<crate_name>.wasm`
and `contracts.txt` containing the hashes. This is just one file.

Run it a few times on different computers
and use `sha256sum` to prove to yourself that this is consistent. I challenge
you to produce a smaller build that works with the cosmwasm integration tests
(and if you do, please make an issue/PR):

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.10.1
```

Demo this with `cosmwasm-examples` (going into eg. `erc20` subdir before running),
with `cosmwasm-plus`, or with a sample app from `cosmwasm-template`.

Note that we use one registry cache (to avoid excessive downloads), but the target cache is a different volume per
contract that we compile. This means no interference between contracts, but very fast recompile times when making
minor adjustments to a contract you had previously created an optimized build for.

## Advanced usage

*This is designed for cosmwasm samples. You cannot provide automatic verification for these*

If you have a more complex build environment, you need to pass a few more
arguments to define how to run the build process.

[`cosmwasm`](https://github.com/CosmWasm/cosmwasm) has a root workspace
and many contracts under `./contracts/*`, which are **excluded** in the
top-level `Cargo.toml`. In this case, we compile each contract separately
with it's own cache. However, since they may refer to packages via path
(`../../packages/std`), we need to run the script in the repo root. In this
case, we can use the optimize.sh command:

```shell
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_burner",target=/code/contracts/burner/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.10.1 ./contracts/burner
```

## Development

Take a look at the [Makefile](https://github.com/CosmWasm/rust-optimizer/blob/master/Makefile)
You can edit the Dockerfile (in a fork), and run `make build` to compile it,
and `make run` to test it (requires the `CODE` env var to be set)

## Notice

This has only been tested on Linux (Ubuntu). There are likely some minor compatibility
issues on OSX and I doubt it runs on Windows without larger changes. If you use one of
those platforms, please submit a PR for better support.
