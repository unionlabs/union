# Copyright 2022 Google LLC
#
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

#!/bin/bash

set -ex
set -o pipefail

readonly PLATFORM="$(uname | tr '[:upper:]' '[:lower:]')"

not() {
  ! "$@"
}

fail_on_output() {
  tee /dev/stderr | not read
}

which go
sudo rm -rf /usr/local/go
case "${PLATFORM}" in
  'linux')
    sudo rm -rf /usr/local/go
    curl -O https://dl.google.com/go/go1.19.12.linux-amd64.tar.gz
    tar -xvf go1.19.12.linux-amd64.tar.gz
    sudo mv go /usr/local
    export GOROOT=/usr/local/go
    export PATH=$PATH:$GOROOT/bin
    ;;
  'darwin')
    sudo rm -rf /usr/local/go
    curl -O https://dl.google.com/go/go1.19.12.darwin-amd64.tar.gz
    tar -xvf go1.19.12.darwin-amd64.tar.gz
    sudo mv go /usr/local
    export GOROOT=/usr/local/go
    export PATH="${GOROOT}/bin:${PATH}"
    ;;
  *)
    echo "Using existing Go installation."
    ;;
esac

go version

# TODO(mattstev): Install goimports and run:
#    goimports -l . 2>&1 | not grep -vE "\.pb\.go"

go vet -all ./... | fail_on_output
gofmt -s -d -l . 2>&1 | fail_on_output
go mod tidy

echo SUCCESS

