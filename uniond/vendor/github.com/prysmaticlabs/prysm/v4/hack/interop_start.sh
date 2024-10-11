#!/bin/bash

"""
2019/09/08 -- Interop start script.
This script is intended for dockerfile deployment for interop testing.
This script is fragile and subject to break as flags change.
Use at your own risk!


Use with interop.Dockerfile from the workspace root:

docker build -f interop.Dockerfile .
"""

# Flags
IDENTITY="" # P2P private key
PEERS="" # Comma separated list of peers
GEN_STATE="" # filepath to ssz encoded state.
PORT="8000" # port to serve p2p traffic
RPCPORT="8001" # port to serve rpc traffic
YAML_KEY_FILE="" # Path to yaml keyfile as defined here: https://github.com/ethereum/eth2.0-pm/tree/master/interop/mocked_start

# Constants
BEACON_LOG_FILE="/tmp/beacon.log"
VALIDATOR_LOG_FILE="/tmp/validator.log"

usage() {
    echo "--identity=<identity>"
    echo "--peer=<peer>"
    echo "--num-validators=<number>"
    echo "--gen-state=<file path>"
    echo "--port=<port number>"
    echo "--rpcport=<port number>"
}

while [ "$1" != "" ];
do
    PARAM=`echo $1 | awk -F= '{print $1}'`
    VALUE=`echo $1 | sed 's/^[^=]*=//g'`

    case $PARAM in
        --identity)
            IDENTITY=$VALUE
            ;;
        --peers)
            [ -z "$PEERS" ] && PEERS+=","
           PEERS+="$VALUE"
            ;;
        --validator-keys)
            YAML_KEY_FILE=$VALUE
            ;;
        --gen-state)
            GEN_STATE=$VALUE
            ;;
        --port)
            PORT=$VALUE
            ;;
        --rpcport)
            RPCPORT=$VALUE
            ;;
        --help)
            usage
            exit
            ;;
        *)
            echo "ERROR: unknown parameter \"$PARAM\""
            usage
            exit 1
            ;;
    esac
    shift
done


echo "Converting hex yaml keys to a format that Prysm understands"

# Expect YAML keys in hex encoded format. Convert this into the format the validator already understands.
./convert-keys $YAML_KEY_FILE /tmp/keys.json

echo "Starting beacon chain and logging to $BEACON_LOG_FILE"

echo -n "$IDENTITY" > /tmp/id.key



BEACON_FLAGS="--bootstrap-node= \
  --deposit-contract=0xD775140349E6A5D12524C6ccc3d6A1d4519D4029 \
  --p2p-port=$PORT \
  --grpc-gateway-port=$RPCPORT \
  --peer=$PEERS \
  --interop-genesis-state=$GEN_STATE \
  --p2p-priv-key=/tmp/id.key \
  --log-file=$BEACON_LOG_FILE"

./beacon-chain $BEACON_FLAGS &

echo "Starting validator client and logging to $VALIDATOR_LOG_FILE"

VALIDATOR_FLAGS="--monitoring-port=9091 \
  --unencrypted-keys /tmp/keys.json \
  --log-file=$VALIDATOR_LOG_FILE

./validator- $VALIDATOR_FLAGS &

