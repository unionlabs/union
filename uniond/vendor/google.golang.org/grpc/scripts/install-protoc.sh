#!/bin/bash
#
# Copyright 2024 gRPC authors.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#      http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#
# install-protoc.sh
#
# This script installs the Protocol Buffers compiler (protoc) to the specified
# directory.  It is used to generate code from .proto files for gRPC
# communication. The script downloads the protoc binary from the official GitHub
# repository and installs it in the system.
#
# Usage: ./install-protoc.sh INSTALL_PATH
#
# Arguments:
#   INSTALL_PATH: The path where the protoc binary will be installed.
#
# Note: This script requires internet connectivity to download the protoc binary.

set -eu -o pipefail

source "$(dirname $0)/common.sh"

# The version of protoc that will be installed.
PROTOC_VERSION="27.1"

main() {
  if [[ "$#" -ne 1 ]]; then
    die "Usage: $0 INSTALL_PATH"
  fi

  INSTALL_PATH="${1}"

  if [[ ! -d "${INSTALL_PATH}" ]]; then
    die "INSTALL_PATH (${INSTALL_PATH}) does not exist."
  fi

  echo "Installing protoc version $PROTOC_VERSION to ${INSTALL_PATH}..."

  # Detect the hardware platform.
  case "$(uname -m)" in
    "x86_64")   ARCH="x86_64";;
    "aarch64")  ARCH="aarch_64";;
    "arm64")    ARCH="aarch_64";;
    *)          die "Install unsuccessful. Hardware platform not supported by installer: $1";;
  esac

  # Detect the Operating System.
  case "$(uname -s)" in
    "Darwin") OS="osx";;
    "Linux")  OS="linux";;
    *)        die "Install unsuccessful. OS not supported by installer: $2";;
  esac

  # Check if the protoc binary with the right version is already installed.
  if [[ -f "${INSTALL_PATH}/bin/protoc" ]]; then
    if [[ "$("${INSTALL_PATH}/bin/protoc" --version)" == "libprotoc ${PROTOC_VERSION}" ]]; then
      echo "protoc version ${PROTOC_VERSION} is already installed in ${INSTALL_PATH}"
      return
    fi
  fi

  DOWNLOAD_URL="https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/protoc-${PROTOC_VERSION}-${OS}-${ARCH}.zip"

  # -L follows redirects.
  # -O writes output to a file.
  curl -LO "${DOWNLOAD_URL}"

  # Unzip the downloaded file and except readme.txt.
  # The file structure should look like:
  # INSTALL_PATH
  # ├── bin
  # │   └── protoc
  # └── include
  #     └── other files...
  unzip "protoc-${PROTOC_VERSION}-${OS}-${ARCH}.zip" -d "${INSTALL_PATH}" -x "readme.txt"
  rm "protoc-${PROTOC_VERSION}-${OS}-${ARCH}.zip"

  # Make the protoc binary executable. ¯\_(ツ)_/¯  crazy, right?
  chmod +x "${INSTALL_PATH}/bin/protoc"
}

main "$@"
