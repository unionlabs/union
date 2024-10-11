#!/usr/bin/env bash

# This script runs unit tests with coverage enabled for a specific list of
# package paths and outputs the coverage to a json file.
#
# Package paths that are not valid in this tree are tolerated.

set -xeuo pipefail

output_json_file="$1"
packages="$2"

# Find the targets. We need to convert from, e.g.
#   . objstorage objstorage/objstorageprovider
# to
#   . ./objstorage ./objstorage/objstorageprovider

paths=""
sep=""

for p in ${packages}; do
  # Check that the path exists and contains Go files.
  if ls "${p}"/*.go >/dev/null 2>&1; then
    if [[ $p != "." ]]; then
      p="./$p"
    fi
    paths="${paths}${sep}${p}"
    sep=" "
  fi
done

if [ -z "${paths}" ]; then
  echo "Skipping"
  touch "${output_json_file}"
  exit 0
fi

tmpfile=$(mktemp --suffix -coverprofile)
trap 'rm -f "${tmpfile}"' EXIT

make testcoverage COVER_PROFILE="${tmpfile}" PKG="$paths"
go run github.com/cockroachdb/code-cov-utils/gocover2json@v1.0.0 \
  --trim-prefix github.com/cockroachdb/pebble/ \
  "${tmpfile}" "${output_json_file}"
