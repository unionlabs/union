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

# Fail on any error.
set -e

# Display commands being run.
set -x

readonly PLATFORM="$(uname | tr '[:upper:]' '[:lower:]')"

fail_with_debug_output() {
  ls -l
  df -h /
  exit 1
}

run_tests() {
  time go build -buildvcs=false ./... || fail_with_debug_output
  time go test -buildvcs=false ./... || fail_with_debug_output
}

main() {
  # Install a newer Golang version on GCP Ubuntu VMs.
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

  run_tests
}

main "$@"
