#!/bin/bash
set -o errexit -o nounset -o pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"

echo "-----------------------"

echo "### List all codes"
RESP=$(grpcurl -plaintext localhost:9090 cosmwasm.wasm.v1.Query/Codes)
echo "$RESP" | jq

CODE_ID=$(echo "$RESP" | jq -r '.codeInfos[-1].codeId')
echo "### List contracts by code"
RESP=$(grpcurl -plaintext -d "{\"codeId\": $CODE_ID}" localhost:9090 cosmwasm.wasm.v1.Query/ContractsByCode)
echo "$RESP" | jq

echo "### Show history for contract"
CONTRACT=$(echo "$RESP" | jq -r ".contracts[-1]")
grpcurl -plaintext -d "{\"address\": \"$CONTRACT\"}" localhost:9090 cosmwasm.wasm.v1.Query/ContractHistory | jq

echo "### Show contract state"
grpcurl -plaintext -d "{\"address\": \"$CONTRACT\"}" localhost:9090 cosmwasm.wasm.v1.Query/AllContractState | jq

echo "Empty state due to 'burner' contract cleanup"
