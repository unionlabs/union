#!/usr/bin/env bash
#
# This file re-generates the sources in this directory from the Go
# standard library.
#
set -euxo pipefail

cp $GOROOT/src/fmt/format.go format.go
patch -p0 <format.go.diff

cp $GOROOT/src/fmt/print.go print.go
patch -p0 <print.go.diff

mkdir -p fmtsort
for a in $GOROOT/src/internal/fmtsort/*; do
    if expr "$a" : ".*_test.go"; then
	continue
    fi
    n=$(basename "$a")
    (
	echo "// Code generated from the Go standard library. DO NOT EDIT"
	echo "// GENERATED FILE DO NOT EDIT"
	cat "$a"
    ) >fmtsort/"$n"
done

