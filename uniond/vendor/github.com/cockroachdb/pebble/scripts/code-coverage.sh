#!/bin/bash

# This script runs unit tests and the metamorphic tests with coverage
# instrumentation and generates three lcov files:
#  - ./artifacts/profile-tests.lcov
#  - ./artifacts/profile-meta.lcov
#  - ./artifacts/profile-tests-and-meta.lcov

set -euxo pipefail

mkdir -p artifacts

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT

test_failed=0
# The coverpkg argument ensures that coverage is not restricted to the tested
# package; so this will get us overall coverage for all tests.
go test -tags invariants ./... -coverprofile=artifacts/profile-tests.gocov -coverpkg=./... || test_failed=1

# The metamorphic test executes itself for each run; we don't get coverage for
# the inner run. To fix this, we use metarunner as the "inner" binary and we
# instrument it with coverage (see https://go.dev/testing/coverage/#building).
go build -tags invariants -o "${tmpdir}/metarunner" -cover ./internal/metamorphic/metarunner
mkdir -p "${tmpdir}/metacover"

GOCOVERDIR="${tmpdir}/metacover" go test ./internal/metamorphic \
  -count 50 --inner-binary="${tmpdir}/metarunner" || test_failed=1

go tool covdata textfmt -i "${tmpdir}/metacover" -o artifacts/profile-meta.gocov

# TODO(radu): make the crossversion metamorphic test work.

go run github.com/cockroachdb/code-cov-utils/convert@v1.1.0 -out artifacts/profile-tests.lcov \
  -trim-prefix github.com/cockroachdb/pebble/ \
  artifacts/profile-tests.gocov

go run github.com/cockroachdb/code-cov-utils/convert@v1.1.0 -out artifacts/profile-meta.lcov \
  -trim-prefix github.com/cockroachdb/pebble/ \
  artifacts/profile-meta.gocov

go run github.com/cockroachdb/code-cov-utils/convert@v1.1.0 -out artifacts/profile-tests-and-meta.lcov \
  -trim-prefix github.com/cockroachdb/pebble/ \
  artifacts/profile-tests.gocov artifacts/profile-meta.gocov

if [ $test_failed -eq 1 ]; then
  # TODO(radu): somehow plumb the error and publish it.
  echo "WARNING: some tests have failed; coverage might be incomplete."
fi
