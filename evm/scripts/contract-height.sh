#!/usr/bin/env bash

set -eou pipefail

ETH_RPC_URL="${RPC_URL:-https://rpc.ankr.com/eth_sepolia/6c72c8d164912bed4694cb882fc4ca55574126511a4f5f66828a53fa2448a20a}"

ETHERSCAN_SEPOLIA_URL="https://api-sepolia.etherscan.io/api"

ETHERSCAN_API_KEY="${ETHERSCAN_API_KEY:-UPIHTDJB5PHJ2MKCND7WFHJNQTFSJG7SSC}"

CONTRACT_ADDRESS="${CONTRACT_ADDRESS:-0xa390514f803a3b318b93bf6cd4beeb9f8299a0eb}"

CONTRACT_CREATION=$(curl --request GET \
  --url "${ETHERSCAN_SEPOLIA_URL}?module=contract&action=getcontractcreation&contractaddresses=${CONTRACT_ADDRESS}&apikey=${ETHERSCAN_API_KEY}" | jq)

CONTRACT_CREATION_TX_HASH=$(echo "${CONTRACT_CREATION}" | jq -r '.result[0].txHash')

echo "Contract creation transaction hash: ${CONTRACT_CREATION_TX_HASH}"

cast tx "${CONTRACT_CREATION_TX_HASH}" --rpc-url "${ETH_RPC_URL}" --json | jq

## all aboce in one line

curl --request GET --url "https://api-sepolia.etherscan.io/api?module=contract&action=getcontractcreation&contractaddresses=0xa390514f803a3b318b93bf6cd4beeb9f8299a0eb&apikey=UPIHTDJB5PHJ2MKCND7WFHJNQTFSJG7SSC" | jq -r '.result[0].txHash' | cast tx --rpc-url https://rpc.ankr.com/eth_sepolia/6c72c8d164912bed4694cb882fc4ca55574126511a4f5f66828a53fa2448a20a --json | jq
