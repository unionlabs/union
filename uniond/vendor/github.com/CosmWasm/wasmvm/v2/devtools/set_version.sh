#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

gnused="$(command -v gsed || echo sed)"

function print_usage() {
  echo "Usage: $0 NEW_VERSION"
  echo ""
  echo "e.g. $0 0.8.0"
}

if [ "$#" -ne 1 ]; then
  print_usage
  exit 1
fi

# Check repo
SCRIPT_DIR="$(realpath "$(dirname "$0")")"
if [[ "$(realpath "$SCRIPT_DIR/..")" != "$(pwd)" ]]; then
  echo "Script must be called from the repo root"
  exit 2
fi

# Ensure repo is not dirty
CHANGES_IN_REPO=$(git status --porcelain)
if [[ -n "$CHANGES_IN_REPO" ]]; then
  echo "Repository is dirty. Showing 'git status' and 'git --no-pager diff' for debugging now:"
  git status && git --no-pager diff
  exit 3
fi

NEW="$1"
echo "Setting version to $NEW ..."

CARGO_TOML="libwasmvm/Cargo.toml"
CARGO_LOCK="libwasmvm/Cargo.lock"
"$gnused" -i -e "s/^version[[:space:]]*=.*/version = \"$NEW\"/" "$CARGO_TOML"
(cd libwasmvm && cargo check && cargo test)
git add "$CARGO_TOML" "$CARGO_LOCK"
git commit -m "Set libwasmvm version: $NEW"
git push

while true; do
  echo "Waiting for library build commit ..."
  sleep 45
  git pull
  if git log --oneline | head -n 1 | grep "[skip ci] Built release libraries"; then
    TAG="v$NEW"
    git tag "$TAG"
    echo "Tag $TAG created. Please review and git push --tags"
    exit 0
  fi
done
