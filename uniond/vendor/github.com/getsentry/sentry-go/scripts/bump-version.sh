#!/bin/bash
set -eux

if [ "$(uname -s)" != "Linux" ]; then
    echo "Please use the GitHub Action."
    exit 1
fi

SCRIPT_DIR="$( dirname "$0" )"
cd $SCRIPT_DIR/..

OLD_VERSION="${1}"
NEW_VERSION="${2}"

echo "Current version: $OLD_VERSION"
echo "Bumping version: $NEW_VERSION"

function replace() {
    ! grep "$2" $3
    perl -i -pe "s!$1!$2!g" $3
    grep "$2" $3  # verify that replacement was successful
}

replace "const SDKVersion = \"[\w.-]+\"" "const SDKVersion = \"$NEW_VERSION\"" ./sentry.go

# Replace root module versions in submodules
GO_MOD_FILES=$(find . -type f -name 'go.mod' -not -path ./go.mod)
for GO_MOD in ${GO_MOD_FILES}; do
    replace "github.com/getsentry/sentry-go v.*" "github.com/getsentry/sentry-go v${NEW_VERSION}" "${GO_MOD}"
done
