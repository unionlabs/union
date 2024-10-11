#!/bin/bash

if [[ "$TRAVIS_GO_VERSION" =~ ^1\.13\. ]] && [[ "$TRAVIS_OS_NAME" == "linux" ]] && [[ "$GO111MODULE" == "on" ]]; then
    $(go env GOPATH)/bin/gox -build-lib
fi
