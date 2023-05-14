#!/bin/bash

set -e

go version

packages=$(go list ./... | grep github.com/dgraph-io/badger/v2/)

if [[ ! -z "$TEAMCITY_VERSION" ]]; then
  export GOFLAGS="-json"
fi

# Ensure that we can compile the binary.
pushd badger
go build -v .
popd

# Run the memory intensive tests first.
go test -v -run='TestBigKeyValuePairs$' --manual=true
go test -v -run='TestPushValueLogLimit' --manual=true

# Run the special Truncate test.
rm -rf p
go test -v -run='TestTruncateVlogNoClose$' --manual=true
truncate --size=4096 p/000000.vlog
go test -v -run='TestTruncateVlogNoClose2$' --manual=true
go test -v -run='TestTruncateVlogNoClose3$' --manual=true
rm -rf p

# Then the normal tests.
echo
echo "==> Starting test for table, skl and y package"
go test -v -race github.com/dgraph-io/badger/v2/skl
# Run test for all package except the top level package. The top level package support the
# `vlog_mmap` flag which rest of the packages don't support.
go test -v -race $packages

echo
echo "==> Starting tests with value log mmapped..."
# Run top level package tests with mmap flag.
go test -timeout=25m -v -race github.com/dgraph-io/badger/v2 --vlog_mmap=true

echo
echo "==> Starting tests with value log not mmapped..."
go test -timeout=25m -v -race github.com/dgraph-io/badger/v2 --vlog_mmap=false

