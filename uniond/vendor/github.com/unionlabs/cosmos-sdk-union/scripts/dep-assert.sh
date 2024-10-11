#!/usr/bin/env bash

set -o errexit

CWD=$(pwd)

# no simapp imports in modules
SIMAPP_REGEX="cosmossdk.io/simapp"
find . -type f -name 'go.mod' -print0 | while IFS= read -r -d '' file
do
  d=$(dirname "$file")
  if [[ "$d" =~ \./simapp$|\./simapp/v2$|\./tests* ]]; then
    continue
  fi

  if cd "$CWD/$d" && go list -test -f '{{ .Imports }}' ./... | grep -q -E "${SIMAPP_REGEX}"; then
    echo "${d} has a dependency on simapp!"
    exit 1
  fi
done

# no runtime/v2 or server/v2 imports in x/ modules
RUNTIMEV2_REGEX="cosmossdk.io/runtime/v2"
SEVERV2_REGEX="cosmossdk.io/server/v2"
XEXP_REGEX="golang.org/x/exp"
find ./x/ -type f -name 'go.mod' -print0 | while IFS= read -r -d '' file
do
  d=$(dirname "$file")
  if cd "$CWD/$d" && go list -test -f '{{ .Imports }}' ./... | grep -q -E "${RUNTIMEV2_REGEX}"; then
    echo "${d} has a dependency on runtime/v2!"
    exit 1
  fi

  if cd "$CWD/$d" && go list -test -f '{{ .Imports }}' ./... | grep -q -E "${SEVERV2_REGEX}"; then
    echo "${d} has a dependency on server/v2!"
    exit 1
  fi

  if cd "$CWD/$d" && go list -test -f '{{ .Imports }}' ./... | grep -q -E "${XEXP_REGEX}"; then
    echo "${d} has a dependency on golang.org/x/exp"
    exit 1
  fi
done