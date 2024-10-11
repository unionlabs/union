#!/bin/bash

set -e

# Ensure relayer is installed
if ! [ -x "$(which rly)" ]; then
  echo "Error: rly is not installed." >&2
  exit 1
fi

rly tx link demo -d

rly tx transfer ibc-0 ibc-1 1000000test $(rly chains address ibc-1)

sleep 2

EXPECTED_BALANCE="100000000000test"
CHAIN_1_BALANCE=$(rly q bal ibc-1)

if [[ "$CHAIN_1_BALANCE" == *"$EXPECTED_BALANCE" ]]; then
    echo "Token not sent correctly"
    echo "$EXPECTED_BALANCE not found in $CHAIN_1_BALANCE"
    exit 1
fi

echo "IBC transfer executed successfully"
