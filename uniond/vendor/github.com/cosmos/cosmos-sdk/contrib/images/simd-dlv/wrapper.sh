#!/usr/bin/env sh
set -euo pipefail
set -x

DEBUG=${DEBUG:-0}
BINARY=/simd/${BINARY:-simd}
ID=${ID:-0}
LOG=${LOG:-simd.log}

if ! [ -f "${BINARY}" ]; then
	echo "The binary $(basename "${BINARY}") cannot be found. Please add the binary to the shared folder. Please use the BINARY environment variable if the name of the binary is not 'simd'"
	exit 1
fi

export SIMDHOME="/data/node${ID}/simd"

if [ "$DEBUG" -eq 1 ]; then
  dlv --listen=:2345 --continue --headless=true --api-version=2 --accept-multiclient exec "${BINARY}" -- --home "${SIMDHOME}" "$@"
elif [ "$DEBUG" -eq 1 ] && [ -d "$(dirname "${SIMDHOME}"/"${LOG}")" ]; then
  dlv --listen=:2345 --continue --headless=true --api-version=2 --accept-multiclient exec "${BINARY}" -- --home "${SIMDHOME}" "$@" | tee "${SIMDHOME}/${LOG}"
elif [ -d "$(dirname "${SIMDHOME}"/"${LOG}")" ]; then
  "${BINARY}" --home "${SIMDHOME}" "$@" | tee "${SIMDHOME}/${LOG}"
else
  "${BINARY}" --home "${SIMDHOME}" "$@"
fi