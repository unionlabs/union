#!/bin/bash
#
# This script verifies that each document in the docs and architecture
# directory has a corresponding table-of-contents entry in its README file.
#
# This can be run manually from the command line.
# It is also run in CI via the docs-toc.yml workflow.
#
set -euo pipefail

readonly base="$(dirname $0)"
cd "$base"

readonly workdir="$(mktemp -d)"
trap "rm -fr -- '$workdir'" EXIT

checktoc() {
    local dir="$1"
    local tag="$2"'-*-*'
    local out="$workdir/${dir}.out.txt"
    (
        cd "$dir" >/dev/null
        find . -maxdepth 1 -type f -name "$tag" -not -exec grep -q "({})" README.md ';' -print
    ) > "$out"
    if [[ -s "$out" ]] ; then
        echo "-- The following files in $dir lack a ToC entry:
"
        cat "$out"
        return 1
    fi
}

err=0

# Verify that each RFC and ADR has a ToC entry in its README file.
checktoc architecture adr || ((err++))
checktoc rfc rfc || ((err++))

exit $err
