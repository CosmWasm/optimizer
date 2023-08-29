#!/usr/bin/env bash

echo "Info: optimize.sh"

set -o errexit -o nounset -o pipefail
trap 'echo >&2 "Error on line $LINENO"' ERR
command -v shellcheck >/dev/null && shellcheck "$0"

export PATH=$PATH:/root/.cargo/bin

# Suffix for non-Intel built artifacts
MACHINE=$(uname -m)
SUFFIX=${MACHINE#x86_64}
SUFFIX=${SUFFIX:+-$SUFFIX}

echo "Info: RUSTC_WRAPPER=$RUSTC_WRAPPER"

echo "Info: sccache stats before build"
sccache -s

mkdir -p artifacts
rm -f artifacts/checksums_intermediate.txt

# There are two cases here
# 1. All contracts (or one) are included in the root workspace (eg. `cosmwasm-template`, `cosmwasm-examples`, `cosmwasm-plus`)
#    In this case, we pass no argument, just mount the proper directory.
# 2. Contracts are excluded from the root workspace, but import relative paths from other packages (only `cosmwasm`).
#    In this case, we mount root workspace and pass in a path `docker run <repo> ./contracts/hackatom`

# This parameter allows us to mount a folder into docker container's "/code"
# and build "/code/contracts/mycontract".
# Note: if CONTRACTDIR is "." (default in Docker), this ends up as a noop
for CONTRACTDIR in "$@"; do
  echo "Building contract in $(realpath "$CONTRACTDIR") ..."
    if [ ! -f "$CONTRACTDIR/Cargo.toml" ]; then
      echo "Cargo.toml not found in $CONTRACTDIR. Skipping this directory."
      continue
    fi
  (
    cd "$CONTRACTDIR"
    echo "Info: Building in $CONTRACTDIR"

    # Get the package name from Cargo.toml
    pkg_name=$(toml get -r Cargo.toml package.name)
    pkg_name=${pkg_name//-/_}

    # Remove all previous artifacts (helps in debugging)
    rm -rf /target/wasm32-unknown-unknown/release/*.wasm

    # Check if there are features
    if toml get Cargo.toml package.metadata.optimizer.features >/dev/null 2>&1; then
         IFS=$'\n' features=($(toml get Cargo.toml package.metadata.optimizer.features | jq -r '.[]'))
    else
        features=()
    fi

    # Build the release for the contract and move it to the artifacts folder
    build_and_move_release() {
      local feature_name=${1:-}
      local feature_flag=""
      if [ -n "$feature_name" ]; then
        feature_flag="--features=${feature_name}"
      fi
      echo "Info: Building with feature: $feature_name"
      RUSTFLAGS='-C link-arg=-s' cargo build --target-dir=/target --release --lib --target wasm32-unknown-unknown --locked ${feature_flag}

      # rename the wasm file (named after the package name) to the feature-specific name (if any).
      local wasm_output="/target/wasm32-unknown-unknown/release/${pkg_name}".wasm
      local wasm_name="/target/wasm32-unknown-unknown/release/${pkg_name}${feature_name:+-$feature_name}".wasm
      mv "$wasm_output" "$wasm_name"
    }

    # Build with features if present
    if [ "${#features[@]}" -gt 0 ]; then
      for feature in "${features[@]}"; do
        build_and_move_release $feature
      done
    fi

    # Build without features after potentially building with features
    build_and_move_release
  )
  
  echo "Info: Finished building in $CONTRACTDIR"

  # wasm-optimize on all results
  for WASM in /target/wasm32-unknown-unknown/release/*.wasm; do
    NAME=$(basename "$WASM" .wasm)${SUFFIX}.wasm
    echo "Creating intermediate hash for $NAME ..."
    sha256sum -- "$WASM" | tee -a artifacts/checksums_intermediate.txt
    echo "Optimizing $NAME ..."
    # --signext-lowering is needed to support blockchains runnning CosmWasm < 1.3. It can be removed eventually
    wasm-opt -Os --signext-lowering "$WASM" -o "artifacts/$NAME"
  done
done

# Create hash
echo "Creating hashes ..."
(
  cd artifacts
  sha256sum -- *.wasm | tee checksums.txt
)

echo "Info: sccache stats after build"
sccache -s

echo "done"
