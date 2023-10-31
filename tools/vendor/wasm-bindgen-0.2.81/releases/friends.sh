#!/usr/bin/env bash

set -eu
set -o pipefail

cd $(dirname $0)

if [[ -z "${1+x}" ]]; then
    read -p "List friends since which commit/tag? " since
    echo
else
    since=$1
fi

git shortlog -s -n "$since.." \
    | cut -f 2- \
    | sort -u \
    | grep -v bors\-servo \
    | xargs -d '\n' -I{} echo "- {}"
