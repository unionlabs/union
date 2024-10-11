#!/bin/bash

set -e

# Install golanci 1.32.2
if [[ "$TRAVIS_GO_VERSION" =~ ^1\.15\. ]]; then
    curl -sfL https://raw.githubusercontent.com/golangci/golangci-lint/master/install.sh| sh -s -- -b $(go env GOPATH)/bin v1.32.2
fi
