#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

function print_usage() {
  echo "Usage: $0 [-h|--help]"
  echo "Publishes crates to crates.io."
}

if [ $# = 1 ] && { [ "$1" = "-h" ] || [ "$1" = "--help" ] ; }
then
    print_usage
    exit 1
fi

# these are imported by other packages
APIS="dex cw-staking tendermint-staking"

 for pack in $APIS; do
   (
     cd "contracts/$pack"
     echo "Publishing base $pack"
     cargo publish
   )
 done

echo "Everything is published!"

VERSION=$(cat Cargo.toml | grep -m 1 version | sed 's/-/_/g' | grep -o '".*"' | sed 's/"//g');
git tag v$VERSION
git push origin v$VERSION
