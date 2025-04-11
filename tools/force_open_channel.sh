#!/bin/sh

CONFIG_PATH="$1"
SOURCE_TYPE=""
DESTINATION_TYPE=""
SOURCE_CLIENT_ID=""
DESTINATION_CLIENT_ID=""
SOURCE_RPC_URL=""
DESTINATION_RPC_URL=""
SOURCE_IBC_CONTRACT=""
DESTINATION_IBC_CONTRACT=""
SOURCE_APP_ADDRESS=""
SOURCE_APP_VERSION=""
SOURCE_RELAYER=""
DESTINATION_APP_ADDRESS=""
DESTINATION_APP_VERSION=""
DESTINATION_RELAYER=""

exit_if_config_cannot_be_parsed() {
  (cat $CONFIG_PATH | jq) > /dev/null || exit 1
}

exit_if_chain_type_not_supported() {
  if [ "$1" != "evm" -a "$1" != "cosmos" ]; then
    echo "chain type not supported, must be \"evm\" or \"cosmos\""
    exit 1
  fi
}

parse_config() {
  SOURCE_TYPE=$(cat $CONFIG_PATH | jq -r .source.type)
  DESTINATION_TYPE=$(cat $CONFIG_PATH | jq -r .destination.type)

  exit_if_chain_type_not_supported $SOURCE_TYPE
  exit_if_chain_type_not_supported $DESTINATION_TYPE

  parse_source_args
  parse_destination_args
}

parse_source_args() {
  SOURCE_CLIENT_ID=$(cat $CONFIG_PATH | jq .source.client_id)

  # TODO check null

  SOURCE_IBC_CONTRACT=$(cat $CONFIG_PATH | jq -r .source.ibc_contract)
  if [[ $SOURCE_IBC_CONTRACT == null ]]; then
    exit "ibc handler cannot be null"
    exit 1
  fi

  SOURCE_RPC_URL=$(cat $CONFIG_PATH | jq -r .source.rpc_url)
  if [[ $SOURCE_RPC_URL == null ]]; then
    exit "source rpc url cannot be null"
    exit 1
  fi

  SOURCE_APP_ADDRESS=$(cat $CONFIG_PATH | jq -r .source.app_contract)
  if [[ $SOURCE_APP_ADDRESS == null ]]; then
    exit "source app contract cannot be null"
    exit 1
  fi

  SOURCE_APP_VERSION=$(cat $CONFIG_PATH | jq -r .source.app_version)
  if [[ $SOURCE_APP_VERSION == null ]]; then
    exit "source app version cannot be null"
    exit 1
  fi


  SOURCE_RELAYER=$(cat $CONFIG_PATH | jq -r .source.relayer)
  if [[ $SOURCE_RELAYER == null ]]; then
    exit "source relayer cannot be null"
    exit 1
  fi
}

parse_destination_args() {
  DESTINATION_CLIENT_ID=$(cat $CONFIG_PATH | jq .destination.client_id)

  # TODO check null

  DESTINATION_IBC_CONTRACT=$(cat $CONFIG_PATH | jq -r .destination.ibc_contract)
  if [[ $DESTINATION_IBC_CONTRACT == null ]]; then
    exit "ibc handler cannot be null"
    exit 1
  fi

  DESTINATION_RPC_URL=$(cat $CONFIG_PATH | jq -r .destination.rpc_url)
  if [[ $DESTINATION_RPC_URL == null ]]; then
    exit "destination rpc url cannot be null"
    exit 1
  fi

  DESTINATION_APP_ADDRESS=$(cat $CONFIG_PATH | jq -r .destination.app_contract)
  if [[ $DESTINATION_APP_ADDRESS == null ]]; then
    exit "destination app contract cannot be null"
    exit 1
  fi

  DESTINATION_APP_VERSION=$(cat $CONFIG_PATH | jq -r .destination.app_version)
  if [[ $DESTINATION_APP_VERSION == null ]]; then
    exit "destination app version cannot be null"
    exit 1
  fi

  DESTINATION_RELAYER=$(cat $CONFIG_PATH | jq -r .destination.relayer)
  if [[ $DESTINATION_RELAYER == null ]]; then
    exit "destination relayer cannot be null"
    exit 1
  fi
}

connection_open_init_evm() {
  PRIVATE_KEY=$(cat $CONFIG_PATH | jq -r .source.private_key)
  if [[ $PRIVATE_KEY == null ]]; then
    exit "private key cannot be null"
    exit 1
  fi

  echo $(cast send $SOURCE_IBC_CONTRACT 'connectionOpenInit((uint32,uint32)) (uint32)' '('$SOURCE_CLIENT_ID,$DESTINATION_CLIENT_ID')' --private-key "$PRIVATE_KEY" -r "$SOURCE_RPC_URL" --json | jq .logs.'[0]'.topics.'[1]' -r | cast to-dec)
}


connection_open_try_cosmos() {
  NODE_HOME=$(cat $CONFIG_PATH | jq -r .destination.home)
  if [[ $NODE_HOME == null ]]; then
    exit "node home cannot be null"
    exit 1
  fi

  KEYNAME=$(cat $CONFIG_PATH | jq -r .destination.keyname)
  if [[ $KEYNAME == null ]]; then
    exit "keyname cannot be null"
    exit 1
  fi

  tx_hash=$(nix run .#uniond -- tx wasm execute $DESTINATION_IBC_CONTRACT \
    "{ \"force_connection_open_try\": { \"counterparty_client_id\": $SOURCE_CLIENT_ID, \"counterparty_connection_id\": $SOURCE_CONNECTION_ID, \"client_id\": $DESTINATION_CLIENT_ID, \"proof_init\": \"0x00\", \"proof_height\": 1 } }" \
     --node $DESTINATION_RPC_URL \
     --home $NODE_HOME \
     --from $KEYNAME \
     --keyring-backend test \
     --gas auto \
     --gas-prices 1muno \
     --gas-adjustment 5.0 \
     --output json \
     -y | jq .txhash -r)

   echo $(nix run .#uniond -- query tx $tx_hash --output json \
       | jq .events \
       | jq  'first(.[] | select(.type=="wasm-connection_open_try")).attributes' \
       | jq 'first(.[] | select(.key=="connection_id")).value' -r)
}

connection_open_ack_evm() {

  PRIVATE_KEY=$(cat $CONFIG_PATH | jq -r .source.private_key)
  if [[ $PRIVATE_KEY == null ]]; then
    exit "private key cannot be null"
    exit 1
  fi

  cast send $SOURCE_IBC_CONTRACT 'forceConnectionOpenAck((uint32,uint32,bytes,uint64))' '('$SOURCE_CONNECTION_ID,$DESTINATION_CONNECTION_ID',0x00,1)' --private-key "$PRIVATE_KEY" -r "$SOURCE_RPC_URL" --json

  echo OPEN ACK
}

connection_open_confirm_cosmos() {
  NODE_HOME=$(cat $CONFIG_PATH | jq -r .destination.home)
  if [[ $NODE_HOME == null ]]; then
    exit "node home cannot be null"
    exit 1
  fi

  KEYNAME=$(cat $CONFIG_PATH | jq -r .destination.keyname)
  if [[ $KEYNAME == null ]]; then
    exit "keyname cannot be null"
    exit 1
  fi

  tx_hash=$(nix run .#uniond -- tx wasm execute $DESTINATION_IBC_CONTRACT \
    "{ \"force_connection_open_confirm\": { \"connection_id\": $DESTINATION_CONNECTION_ID, \"proof_ack\": \"0x00\", \"proof_height\": 1 } }" \
     --node $DESTINATION_RPC_URL \
     --home $NODE_HOME \
     --from $KEYNAME \
     --keyring-backend test \
     --gas auto \
     --gas-prices 1muno \
     --gas-adjustment 5.0 \
     --output json \
     -y | jq .txhash -r)

   echo OPEN CONFIRM: $tx_hash
}

# channel_open_init_evm() {
#   PRIVATE_KEY=$(cat $CONFIG_PATH | jq -r .source.private_key)
#   if [[ $PRIVATE_KEY == null ]]; then
#     exit "private key cannot be null"
#     exit 1
#   fi

#   dest_app=$(printf "%s" `printf '%s' $DESTINATION_APP_ADDRESS | xxd -p -u`)

#   echo $(cast send $SOURCE_IBC_CONTRACT \
#     'channelOpenInit((address,bytes,uint32,string,address)) (uint32)' \
#     '('$SOURCE_APP_ADDRESS,$dest_app,$SOURCE_CONNECTION_ID,$SOURCE_APP_VERSION,$SOURCE_RELAYER')' \
#     --private-key "$PRIVATE_KEY" \
#     -r "$SOURCE_RPC_URL" \
#     --json \
#     | jq .logs.'[0]'.topics.'[2]' -r \
#     | cast to-dec)
# }


# channel_open_try_cosmos() {
#   NODE_HOME=$(cat $CONFIG_PATH | jq -r .destination.home)
#   if [[ $NODE_HOME == null ]]; then
#     exit "node home cannot be null"
#     exit 1
#   fi

#   KEYNAME=$(cat $CONFIG_PATH | jq -r .destination.keyname)
#   if [[ $KEYNAME == null ]]; then
#     exit "keyname cannot be null"
#     exit 1
#   fi

#   echo nix run .#uniond -- tx wasm execute $DESTINATION_IBC_CONTRACT \
#     "{ \"force_channel_open_try\": { \"port_id\": \"$DESTINATION_APP_ADDRESS\", \"channel\": { \"state\": 1, \"connection_id\": $DESTINATION_CONNECTION_ID, \"counterparty_port_id\": \"$SOURCE_APP_ADDRESS\", \"version\": \"$DESTINATION_APP_VERSION\"}, \"counterparty_version\": \"$SOURCE_APP_VERSION\", \"proof_init\": \"0x00\", \"proof_height\": 1, \"relayer\": \"$DESTINATION_RELAYER\" } }" \
#      --node $DESTINATION_RPC_URL \
#      --home $NODE_HOME \
#      --from $KEYNAME \
#      --keyring-backend test \
#      --gas auto \
#      --gas-prices 1muno \
#      --gas-adjustment 5.0 \
#      --output json \
#      -y # | jq .txhash -r)

#    echo $(nix run .#uniond -- query tx $tx_hash --output json \
#        | jq .events \
#        | jq  'first(.[] | select(.type=="wasm-channel_open_try")).attributes' \
#        | jq 'first(.[] | select(.key=="channel_id")).value' -r)
# }

# connection_open_ack_evm() {

#   PRIVATE_KEY=$(cat $CONFIG_PATH | jq -r .source.private_key)
#   if [[ $PRIVATE_KEY == null ]]; then
#     exit "private key cannot be null"
#     exit 1
#   fi

#   cast send $SOURCE_IBC_CONTRACT 'forceConnectionOpenAck((uint32,uint32,bytes,uint64))' '('$SOURCE_CONNECTION_ID,$DESTINATION_CONNECTION_ID',0x00,1)' --private-key "$PRIVATE_KEY" -r "$SOURCE_RPC_URL" --json

#   echo OPEN ACK
# }

# connection_open_confirm_cosmos() {
#   NODE_HOME=$(cat $CONFIG_PATH | jq -r .destination.home)
#   if [[ $NODE_HOME == null ]]; then
#     exit "node home cannot be null"
#     exit 1
#   fi

#   KEYNAME=$(cat $CONFIG_PATH | jq -r .destination.keyname)
#   if [[ $KEYNAME == null ]]; then
#     exit "keyname cannot be null"
#     exit 1
#   fi

#   tx_hash=$(nix run .#uniond -- tx wasm execute $DESTINATION_IBC_CONTRACT \
#     "{ \"force_connection_open_confirm\": { \"connection_id\": $DESTINATION_CONNECTION_ID, \"proof_ack\": \"0x00\", \"proof_height\": 1 } }" \
#      --node $DESTINATION_RPC_URL \
#      --home $NODE_HOME \
#      --from $KEYNAME \
#      --keyring-backend test \
#      --gas auto \
#      --gas-prices 1muno \
#      --gas-adjustment 5.0 \
#      --output json \
#      -y | jq .txhash -r)

#    echo OPEN CONFIRM: $tx_hash
# }



exit_if_config_cannot_be_parsed

parse_config

SOURCE_CONNECTION_ID=$(connection_open_init_$SOURCE_TYPE)
echo SOURCE $SOURCE_CONNECTION_ID
DESTINATION_CONNECTION_ID=$(connection_open_try_$DESTINATION_TYPE)
echo DESTINATION $DESTINATION_CONNECTION_ID

connection_open_ack_$SOURCE_TYPE
connection_open_confirm_$DESTINATION_TYPE

# SOURCE_CHANNEL_ID=$(channel_open_init_$SOURCE_TYPE)
# echo SOURCE CHANNEL $SOURCE_CHANNEL_ID
# DESTINATION_CHANNEL_ID=$(channel_open_try_$DESTINATION_TYPE)
# echo DESTINATION CHANNEL $DESTINATION_CHANNEL_ID
