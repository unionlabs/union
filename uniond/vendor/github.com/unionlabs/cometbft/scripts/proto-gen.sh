#!/bin/sh
#
# Update the generated code for protocol buffers in the CometBFT repository.
# This must be run from inside a CometBFT working directory.
#
set -euo pipefail

# Work from the root of the repository.
cd "$(git rev-parse --show-toplevel)"

# Run inside Docker to install the correct versions of the required tools
# without polluting the local system.
docker run --rm -i -v "$PWD":/w --workdir=/w golang:1.22-alpine sh <<"EOF"
apk add git make

go install github.com/bufbuild/buf/cmd/buf
go install github.com/cosmos/gogoproto/protoc-gen-gocosmos@latest
make proto-gen
EOF
