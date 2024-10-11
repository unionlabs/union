#!/bin/sh

find_files() {
  find . ! \( \
      \( \
        -path '.github' \
        -o -path './build/_workspace' \
        -o -path './build/bin' \
        -o -path './crypto/bn256' \
        -o -path '*/vendor/*' \
      \) -prune \
    \) -name '*.go'
}

GOFMT="gofmt -s -w"
GOIMPORTS="goimports -w"
find_files | xargs $GOFMT
find_files | xargs $GOIMPORTS
