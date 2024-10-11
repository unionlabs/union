#!/usr/bin/env bash

set -e

REPO_ROOT="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )/.." &> /dev/null && pwd )"
export REPO_ROOT

LINT_TAGS="e2e,ledger,test_ledger_mock"
if [[ ! -z "${ROCKSDB:-}" ]]; then
  LINT_TAGS+=",rocksdb"
fi
export LINT_TAGS

lint_module() {
  local root="$1"
  shift
  if [ -f $root ]; then
    cd "$(dirname "$root")"
  else
    cd "$REPO_ROOT/$root"
  fi
  echo "linting $(grep "^module" go.mod) [$(date -Iseconds -u)]"
  golangci-lint run ./... -c "${REPO_ROOT}/.golangci.yml" "$@" --build-tags=${LINT_TAGS}

  # always lint simapp with app_v1 build tag, otherwise it never gets linted
  if [[ "$(grep "^module" go.mod)" == "module cosmossdk.io/simapp" ]]; then
    golangci-lint run ./... -c "${REPO_ROOT}/.golangci.yml" "$@" --build-tags=app_v1
  fi
}
export -f lint_module

# if LINT_DIFF env is set, only lint the files in the current commit otherwise lint all files
if [[ -z "${LINT_DIFF:-}" ]]; then
  find "${REPO_ROOT}" -type f -name go.mod -print0 | xargs -0 -I{} bash -c 'lint_module "$@"' _ {} "$@"
else
  if [[ -z $GIT_DIFF ]]; then
    GIT_DIFF=$(git diff --name-only) || true
  fi

  if [[ -z "$GIT_DIFF" ]]; then
    echo "no files to lint"
    exit 0
  fi

  GIT_DIFF=$(echo $GIT_DIFF | tr -d "'" | tr ' ' '\n' | grep '\.go$' | grep -v '\.pb\.go$' | grep -Eo '^[^/]+\/[^/]+' | uniq)

  lint_sdk=false
  for dir in ${GIT_DIFF[@]}; do
    if [[ ! -f "$REPO_ROOT/$dir/go.mod" ]]; then
      lint_sdk=true
    else
      lint_module $dir "$@"
    fi
  done

  if [[ $lint_sdk ]]; then
    cd "$REPO_ROOT"
    echo "linting github.com/cosmos/cosmos-sdk [$(date -Iseconds -u)]"
    golangci-lint run ./... -c "${REPO_ROOT}/.golangci.yml" "$@" --build-tags=${LINT_TAGS}
  fi
fi