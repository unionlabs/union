#!/usr/bin/env bash

set -e -o pipefail

REPO_ROOT="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )/.." &> /dev/null && pwd )"
export REPO_ROOT

lint_module() {
  local root="$1"
  shift
  cd "$(dirname "$root")" &&
    echo "linting $(grep "^module" go.mod) [$(date -Iseconds -u)]" &&
    golangci-lint run ./... -c "${REPO_ROOT}/.golangci.yml" "$@"
}
export -f lint_module

find "${REPO_ROOT}" -type f -name go.mod -print0 |
  xargs -0 -I{} bash -c 'lint_module "$@"' _ {} "$@"