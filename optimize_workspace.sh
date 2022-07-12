#!/bin/ash
# shellcheck shell=dash
# See https://www.shellcheck.net/wiki/SC2187
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

export PATH="$PATH:/root/.cargo/bin"

# Suffix for non-Intel built artifacts
MACHINE=$(uname -m)
SUFFIX=${MACHINE#x86_64}
SUFFIX=${SUFFIX:+-$SUFFIX}

rustup toolchain list
cargo --version

# Delete already built artifacts
rm -f target/wasm32-unknown-unknown/release/*.wasm

# Build artifacts
echo "Building artifacts in workspace..."
/usr/local/bin/build_workspace

mkdir -p artifacts
echo "Optimizing artifacts in workspace..."
TMPARTIFACTS=$(mktemp -p "$(pwd)" -d artifacts.XXXXXX)
# Optimize artifacts
(
  cd "$TMPARTIFACTS"

  for WASM in ../target/wasm32-unknown-unknown/release/*.wasm; do
    BASENAME=$(basename "$WASM" .wasm)
    NAME=${BASENAME}${SUFFIX}
    OPTIMIZED_WASM=${NAME}.wasm

    INTERMEDIATE_SHA=$(sha256sum -- "$WASM" | sed 's,../target,target,g')
    INTERMEDIATE_SHAS="../artifacts/checksums_intermediate.txt"

    OPTIMIZATION_EXISTS=1
    if test -f "../artifacts/${OPTIMIZED_WASM}"; then
      OPTIMIZED_SHA=$(sha256sum -- "../artifacts/$OPTIMIZED_WASM" | sed 's,../artifacts/,,g')
      OPTIMIZED_SHAS="../artifacts/checksums.txt"
      grep -Fxq "$OPTIMIZED_SHA" "$OPTIMIZED_SHAS"
      OPTIMIZATION_EXISTS=$?
    fi

    if grep -Fxq "$INTERMEDIATE_SHA" "$INTERMEDIATE_SHAS" && test "$OPTIMIZATION_EXISTS" -eq 0; then
      echo "$BASENAME unchanged. Skipping optimization."
    else
      grep -vs "$BASENAME" "$INTERMEDIATE_SHAS" >tmp_shas && mv -f tmp_shas "$INTERMEDIATE_SHAS"
      echo "Creating intermediate hash for ${BASENAME}..."
      echo "$INTERMEDIATE_SHA" | tee -a "$INTERMEDIATE_SHAS" >/dev/null
      echo "Optimizing ${BASENAME}..."
      wasm-opt -Os "$WASM" -o "$OPTIMIZED_WASM"
      echo "Moving wasm file..."
      mv "$OPTIMIZED_WASM" ../artifacts
    fi
  done
)
rm -rf "$TMPARTIFACTS"
echo "Post-processing artifacts in workspace..."
(
  cd artifacts
  sha256sum -- *.wasm | tee checksums.txt
)

echo "done"
