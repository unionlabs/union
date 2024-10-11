#!/usr/bin/env bash
set -Eeuo pipefail

# Search for rocksdb_version in makefile
rocksdb_version=$(grep "rocksdb_version" ./scripts/build/build.mk | cut -d'=' -f2)

if [[ -z "${rocksdb_version}" ]]; then
    echo "Error: rocksdb_version not found in ./scripts/build/build.mk" >&2
    exit 1
else
    echo "ROCKSDB_VERSION=${rocksdb_version}" >> "${GITHUB_ENV}"
fi