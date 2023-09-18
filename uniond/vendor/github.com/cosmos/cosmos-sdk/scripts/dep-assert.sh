#!/usr/bin/env bash

set -o errexit

SIMAPP_REGEX="cosmossdk.io/simapp"
CWD=$(pwd)


find . -type f -name 'go.mod' -print0 | while IFS= read -r -d '' file
do
  d=$(dirname "$file")
  if [[ "$d" =~ \./simapp$|\./tests$ ]]; then
    continue
  fi

  if cd "$CWD/$d" && go list -test -f '{{ .Imports }}' ./... | grep -q -E "${SIMAPP_REGEX}"; then
    echo "${d} has a dependency on simapp!"
    exit 1
  fi
done
