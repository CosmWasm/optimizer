# Cosmwasm Optimizing Compiler

[![DockerHub](https://img.shields.io/docker/pulls/confio/cosmwasm-opt?style=plastic)](https://hub.docker.com/r/confio/cosmwasm-opt)

This is a Docker build with a locked set of dependencies to produce
reproducible builds of cosmwasm smart contracts. It also does heavy
optimization on the build size, using binary stripping and `wasm-opt`.

## Usage

The easiest way is to simply use the [published docker image](https://hub.docker.com/r/confio/cosmwasm-opt).
You must set the local path to the smart contract you wish to compile and
it will produce a `contract.wasm` file in the same directory.

Run it a few times on different computers
and use `sha256sum` to prove to yourself that this is consistent. I challenge
you to produce a smaller build that works with the cosmwasm integration tests
(and if you do, please make an issue/PR):

```sh
docker run --rm -v $(pwd):/code \
  --mount type=volume,source=$(basename $(pwd))_cache,target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  confio/cosmwasm-opt:0.7.2
```

Note that we use one registry cache (to avoid excessive downloads), but the target cache is a different volume per
contract that we compile. This means no interference between contracts, but very fast recompile times when making
minor adjustments to a contract you had previously created an optimized build for.

## Development

Take a look at the [Makefile](https://github.com/confio/cosmwasm-opt/blob/master/Makefile)
You can edit the Dockerfile (in a fork), and run `make build` to compile it,
and `make run` to test it (requires the `CODE` env var to be set)

## Notice

This has only been tested on Linux (Ubuntu). There are likely some minor compatibility
issues on OSX and I doubt it runs on Windows without larger changes. If you use one of
those platforms, please submit a PR for better support.
