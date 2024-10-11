#!/bin/bash

set -ex  # Exit on error; debugging enabled.
set -o pipefail  # Fail a pipe if any sub-command fails.

# - Source them sweet sweet helpers.
source "$(dirname $0)/common.sh"

# - Check to make sure it's safe to modify the user's git repo.
git status --porcelain | fail_on_output

# - Undo any edits made by this script.
cleanup() {
  git reset --hard HEAD
}
trap cleanup EXIT

# - Installs protoc into your ${GOBIN} directory, if requested.
# ($GOBIN might not be the best place for the protoc binary, but is at least
# consistent with the place where all binaries installed by scripts in this repo
# go.)
if [[ "$1" = "-install" ]]; then
    if [[ "${GITHUB_ACTIONS}" = "true" ]]; then
      source ./scripts/install-protoc.sh "/home/runner/go"
    else
      die "run protoc installer https://github.com/grpc/grpc-go/blob/master/scripts/install-protoc.sh"
    fi
  echo SUCCESS
  exit 0
elif [[ "$#" -ne 0 ]]; then
  die "Unknown argument(s): $*"
fi

# - Check that generated proto files are up to date.
go generate google.golang.org/grpc/... && git status --porcelain 2>&1 | fail_on_output || \
(git status; git --no-pager diff; exit 1)

echo SUCCESS
exit 0
