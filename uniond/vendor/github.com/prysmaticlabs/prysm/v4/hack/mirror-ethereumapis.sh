#!/usr/bin/env bash
#
# Script to mirror a tag from Prysm into EthereumAPIs protocol buffers
#
# Example:
#
# mirror-ethereumapis.sh
#
set -e

# Validate settings.
[ "$TRACE" ] && set -x

## Define variables.
GH_API="https://api.github.com"
GH_REPO="$GH_API/repos/prysmaticlabs/ethereumapis"

AUTH="Authorization: token $GITHUB_SECRET_ACCESS_TOKEN"
## skipcq: SH-2034
export WGET_ARGS="--content-disposition --auth-no-challenge --no-cookie"
## skipcq: SH-2034
export CURL_ARGS="-LJO#"

## Validate token.
curl -o /dev/null -sH "$AUTH" "$GH_REPO" || { echo "Error: Invalid repo, token or network issue!";  exit 1; }

# Clone ethereumapis and prysm
git clone https://github.com/prysmaticlabs/prysm /tmp/prysm/
git clone https://github.com/prysmaticlabs/ethereumapis /tmp/ethereumapis/

# Checkout the release tag in prysm and copy over protos
cd /tmp/prysm && git checkout "$BUILDKITE_COMMIT"

# Copy proto files, go files, and markdown files
find proto/eth \( -name '*.go' -o -name '*.proto' -o -name '*.md' \) -print0 |
    while IFS= read -r -d '' line; do
        item_path=$(dirname "$line")
        mkdir -p /tmp/ethereumapis"${item_path#*proto}" && cp "$line" /tmp/ethereumapis"${line#*proto}"
    done

cd /tmp/ethereumapis || exit

## Replace imports in go files and proto files as needed
find ./eth -name '*.go' -print0 |
    while IFS= read -r -d '' line; do
        sed -i 's/prysm\/proto\/eth/ethereumapis\/eth/g' "$line"
    done

find ./eth -name '*.go' -print0 |
    while IFS= read -r -d '' line; do
        sed -i 's/proto\/eth/eth/g' "$line"
    done

find ./eth -name '*.go' -print0 |
    while IFS= read -r -d '' line; do
        sed -i 's/proto_eth/eth/g' "$line"
    done

find ./eth -name '*.proto' -print0 |
    while IFS= read -r -d '' line; do
        sed -i 's/"proto\/eth/"eth/g' "$line"
    done

find ./eth -name '*.proto' -print0 |
    while IFS= read -r -d '' line; do
        sed -i 's/prysmaticlabs\/prysm\/proto\/eth/prysmaticlabs\/ethereumapis\/eth/g' "$line"
    done

if git diff-index --quiet HEAD --; then
   echo "nothing to push, exiting early"
   exit 0
else
   echo "changes detected, committing and pushing to ethereumapis"
fi

# Push to the mirror repository
git add --all
GIT_AUTHOR_EMAIL=contact@prysmaticlabs.com GIT_AUTHOR_NAME=prysm-bot GIT_COMMITTER_NAME=prysm-bot GIT_COMMITTER_EMAIL=contact@prysmaticlabs.com git commit -am "Mirrored from github.com/prysmaticlabs/prysm@$BUILDKITE_COMMIT"
git remote set-url origin https://prylabs:"$GITHUB_SECRET_ACCESS_TOKEN"@github.com/prysmaticlabs/ethereumapis.git
git push origin master
