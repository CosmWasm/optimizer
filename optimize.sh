#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck > /dev/null && shellcheck "$0"

export PATH=$PATH:/root/.cargo/bin

contracts_root="$1"

# See https://stackoverflow.com/a/23357277/2013738 to understand this find-to-array monster
contract_dirs=()
while IFS=  read -r -d $'\0'; do
  contract_dir=$(dirname "$REPLY")
  contract_dirs+=("$contract_dir")
done < <(find "$contracts_root" \( -name target -o -name package -o -name .git \) -type d -prune -false -o -name Cargo.toml -type f -print0)
echo "Found the following contract dirs: ${contract_dirs[*]}"

for contract_dir in "${contract_dirs[@]}"; do
  echo "Building contract in $(realpath -m "$contract_dir")"
  (
    cd "$contract_dir"

    # Linker flag "-s" for stripping (https://github.com/rust-lang/cargo/issues/3483#issuecomment-431209957)
    # Note that shortcuts from .cargo/config are not available in source code packages from crates.io
    RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown --locked
  )
done

mkdir -p artifacts
artifacts_dir=$(realpath artifacts)

# find the target directories and build results, optimize and write to artifacts
while IFS= read -r -d '' target_dir; do
  for wasm in "$target_dir"/release/*.wasm; do
    echo "Found build result $wasm"
    name=$(basename "$wasm")
    wasm-opt -Os "$wasm" -o "$artifacts_dir/$name"
    echo "Created artifact $name"
  done
done <  <(find . -name wasm32-unknown-unknown -type d -print0)

# create hash
(
  cd "$artifacts_dir"
  sha256sum -- *.wasm > checksums.txt
)

echo "done"
