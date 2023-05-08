#!/bin/bash

set -o errexit -o nounset -o pipefail
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

for CONTRACTDIR in "$@"; do
  echo "Building contract in $(realpath "$CONTRACTDIR") ..."
  (
    cd "$CONTRACTDIR"
    echo "Info: Building in $CONTRACTDIR"

    pkg_name=$(yq e '.package.name' Cargo.toml -o y)
    pkg_name=${pkg_name//-/_}

    features=($(yq e '.package.metadata.optimizer.features.[]' Cargo.toml -o csv))

    # Build the release for the contract and move it to the artifacts folder
    build_and_move_release() {
      local feature_flag=${1:-}
      RUSTFLAGS='-C link-arg=-s' cargo build --release --lib --target wasm32-unknown-unknown --locked ${feature_flag}
      local wasm_output="./target/wasm32-unknown-unknown/release/${pkg_name}".wasm
      local wasm_name="./target/wasm32-unknown-unknown/release/${pkg_name}${feature_flag:+-}${feature_flag}".wasm
      mv "$wasm_output" "$wasm_name"
    }

    # Build without features
    build_and_move_release

    # Build with features if present
    if [ "${#features[@]}" -gt 0 ]; then
      for feature in "${features[@]}"; do
        echo "Building with feature: $feature"
        build_and_move_release "--features $feature"
      done
    fi
  )

  echo "Info: Finished building in $CONTRACTDIR"

  # wasm-optimize on all results
  for WASM in "$CONTRACTDIR"/target/wasm32-unknown-unknown/release/*.wasm; do
    NAME=$(basename "$WASM" .wasm)${SUFFIX}.wasm
    echo "Creating intermediate hash for $NAME ..."
    sha256sum -- "$WASM" | tee -a artifacts/checksums_intermediate.txt
    echo "Optimizing $NAME ..."
    wasm-opt -Os "$WASM" -o "artifacts/$NAME"
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
