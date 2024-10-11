#!/bin/bash

# Copyright 2022 Google LLC.
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     https://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

set -eu

# Create a folder to hold the binaries
rm -rf ./build/bin/darwin_arm64
mkdir -p ./build/bin/darwin_arm64

# Build the signer binary
cd ./internal/signer/darwin
CGO_ENABLED=1 GO111MODULE=on GOARCH=arm64 go build
mv darwin ./../../../build/bin/darwin_arm64/ecp
cd ./../../..

# Build the signer library
CGO_ENABLED=1 GO111MODULE=on GOARCH=arm64 go build -buildmode=c-shared -o build/bin/darwin_arm64/libecp.dylib cshared/main.go
rm build/bin/darwin_arm64/libecp.h
