#!/usr/bin/env bash

set -euo pipefail

# run from the root of the project

nix build .#evm-contracts --print-build-logs

IBC_HANDLER=$(jq --slurp 'map(.abi) | add' \
    result/out/IBCClient.sol/IBCClient.json \
    result/out/IBCPacket.sol/IBCPacket.json \
    result/out/IBCConnection.sol/IBCConnection.json \
    result/out/OwnableIBCHandler.sol/OwnableIBCHandler.json \
    result/out/IBCChannelHandshake.sol/IBCChannelHandshake.json)

echo "export const ibcHandlerAbi = <const>${IBC_HANDLER}" >| app/src/lib/abi/ibc-handler.ts

UCS01_RELAY=$(jq <result/out/Relay.sol/UCS01Relay.json '.abi')

echo "export const ucs01RelayAbi = <const>${UCS01_RELAY}" >| app/src/lib/abi/ucs01-relay.ts
