#!/bin/bash

# Copyright 2020 Google LLC.
# Use of this source code is governed by a BSD-style
# license that can be found in the LICENSE file.

set -e -x

cd $(dirname $0)/../../..

tempdir=$(mktemp -d)
sudo apt-get update && sudo apt-get install -y jq
GOVERSION=$(curl https://golang.org/dl/?mode=json | jq ".[0].version" | sed s/\"//g) &&
    curl -o /tmp/go.tgz https://dl.google.com/go/${GOVERSION}.linux-amd64.tar.gz &&
    tar -C $tempdir -xzf /tmp/go.tgz &&
    rm /tmp/go.tgz &&
    export PATH=$tempdir/go/bin:$PATH &&
    export GOROOT=$tempdir/go

go version
go env

pushd google-api-go-generator
make all
popd
