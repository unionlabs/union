#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

if [ $# -ne 1 ]; then
  echo "Usage: ./download_releases.sh RELEASE_TAG"
  exit 1
fi

tag="$1"

for contract in cyberpunk hackatom queue reflect ibc_reflect; do
  url="https://github.com/CosmWasm/cosmwasm/releases/download/$tag/$contract.wasm"
  echo "Downloading $url ..."
  wget -O "$contract.wasm" "$url"
done
