#!/bin/bash
# This script is used to ensure that the go.mod file is up to date.

set -euo pipefail

for i in $(find $PWD -name go.mod); do
    pushd $(dirname $i)
    go mod tidy
    popd
done 

if [ ! -z "$(git status --porcelain)" ]; then
    git status
    git diff
    echo
    echo "The go.mod is not up to date."
    exit 1
fi
