#!/usr/bin/env bash
#
# This is a convenience script that takes a list of testnet manifests
# as arguments and runs each one of them sequentially. If a testnet
# fails, the container logs are dumped to stdout along with the testnet
# manifest, but the remaining testnets are still run.
#
# This is mostly used to run generated networks in nightly CI jobs.
#

set -euo pipefail

if [[ $# == 0 ]]; then
	echo "Usage: $0 [MANIFEST...]" >&2
	exit 1
fi

FAILED=()

for MANIFEST in "$@"; do
	START=$SECONDS
	echo "==> Running testnet: $MANIFEST"

	if ! ./build/runner -f "$MANIFEST"; then
		echo "==> Testnet $MANIFEST failed, dumping manifest..."
		cat "$MANIFEST"

		echo "==> Dumping container logs for $MANIFEST..."
		./build/runner -f "$MANIFEST" logs

		echo "==> Cleaning up failed testnet $MANIFEST..."
		./build/runner -f "$MANIFEST" cleanup

		FAILED+=("$MANIFEST")
	fi

	echo "==> Completed testnet $MANIFEST in $(( SECONDS - START ))s"
	echo ""
done

if [[ ${#FAILED[@]} -ne 0 ]]; then
	echo "${#FAILED[@]} testnets failed:"
	for MANIFEST in "${FAILED[@]}"; do
		echo "- $MANIFEST"
	done
	exit 1
else
	echo "All testnets successful"
fi
