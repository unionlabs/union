_: {
  perSystem =
    {
      devnetConfig,
      pkgs,
      self',
      cw-instantiate2-salt,
      ...
    }:
    { };
  # let
  #   uniond = pkgs.lib.getExe self'.packages.uniond;

  #   # Name of the account key that we use for the devnet
  #   keyName = "testkey";
  #   chainId = "union-devnet-1";

  #   accountAddress = ''
  #     ${uniond} keys show ${keyName} \
  #       --keyring-backend test \
  #       --address \
  #       --home ${self'.packages.devnet-genesis}
  #   '';

  #   instantiateContract = { code-id, label }:
  #     ''
  #       ACCOUNT_ADDRESS="$(${accountAddress})"

  #       echo ------------------------------------
  #       echo + Instantiating ${label}:
  #       echo     - Message: "$(echo "$INIT_MESSAGE" | jq)"
  #       echo ------------------------------------

  #       while ! ${uniond} tx wasm instantiate2 ${toString code-id} \
  #         "$INIT_MESSAGE" \
  #         ${cw-instantiate2-salt} \
  #         --label ${label} \
  #         --gas=auto \
  #         --gas-adjustment=1.3 -y  \
  #         --admin "$ACCOUNT_ADDRESS" \
  #         --keyring-backend test \
  #         --from ${keyName} \
  #         --chain-id ${chainId} \
  #         --home ${self'.packages.devnet-genesis} > /dev/null
  #       do
  #         echo "Chain doesn't seem to be ready yet. Will retry in 3 seconds."
  #         sleep 3
  #       done
  #     '';

  #   instantiateCwUCS01 =
  #     pkgs.writeShellApplication {
  #       name = "instantiate-ucs01-relay";
  #       runtimeInputs = [ ];
  #       text =
  #         ''
  #           # This account will be the governor and admin of the contract that we instantiate
  #           ACCOUNT_ADDRESS="$(${accountAddress})"

  #           INIT_MESSAGE='{
  #               "default_timeout":300,
  #               "gov_contract": "'"$ACCOUNT_ADDRESS"'",
  #               "channel":{
  #                 "endpoint":{
  #                   "port_id": "",
  #                   "channel_id":"channel-0"
  #                 },
  #                 "counterparty_endpoint":{
  #                   "port_id":"transfer",
  #                  "channel_id":"channel-0"
  #                 },
  #                 "order":"ORDER_UNORDERED",
  #                 "version":"ucs01-0",
  #                 "connection_id":"connection-0"
  #               }
  #             }'

  #           echo "$INIT_MESSAGE"

  #           ${instantiateContract { code-id = 1; label = "ucs01-relay"; }}
  #         '';
  #     };

  #   instantiatePingPong =
  #     pkgs.writeShellApplication {
  #       name = "instantiate-ping-pong";
  #       runtimeInputs = [ ];
  #       text =
  #         ''
  #           TIMEOUT=$1
  #           INIT_MESSAGE='{
  #               "config": {
  #                     "number_of_block_before_pong_timeout": '"$TIMEOUT"',
  #                     "revision_number": 1
  #                 }
  #             }'

  #           ${instantiateContract { code-id = 2; label = "ping-pong"; }}
  #         '';
  #     };
  # in
  # {
  #   packages.e2e-setup =
  #     pkgs.writeShellApplication {
  #       name = "union-devnet-demo";
  #       runtimeInputs = [ pkgs.jq ];
  #       text = ''
  #         # Kill all subprocesses the root process dies
  #         trap "kill 0" EXIT

  #         DEFAULT_GALOIS_URL="http://0.0.0.0:16657"
  #         DEFAULT_EVM_BEACON_RPC_URL="http://localhost:9596"
  #         DEFAULT_EVM_WS_URL="ws://localhost:8546"
  #         DEFAULT_UNION_RPC_URL="http://localhost:26657"
  #         DEFAULT_UNION_GRPC_URL="http://localhost:9090"
  #         DEFAULT_UNION_WS_URL="ws://localhost:26657/websocket"
  #         DEFAULT_PING_PONG_TIMEOUT=1000

  #         HANDSHAKE=""
  #         GALOIS_URL="$DEFAULT_GALOIS_URL"
  #         GALOIS_TLS=""
  #         CIRCUIT_PATH=""
  #         EVM_BEACON_RPC_URL="$DEFAULT_EVM_BEACON_RPC_URL"
  #         EVM_WS_URL="$DEFAULT_EVM_WS_URL"
  #         UNION_RPC_URL="$DEFAULT_UNION_RPC_URL"
  #         UNION_WS_URL="$DEFAULT_UNION_WS_URL"
  #         UNION_GRPC_URL="$DEFAULT_UNION_GRPC_URL"
  #         VOYAGER_CONFIG_FILE=""
  #         UNION_DUMP_PATH=""
  #         NO_DEPLOY_CONTRACTS=""
  #         PING_PONG_MODULE_ADDRESS=""
  #         PING_PONG_TIMEOUT="$DEFAULT_PING_PONG_TIMEOUT"
  #         NO_RUN_VOYAGER=""

  #         printHelp() {
  #           printf " \
  #             Usage: e2e-setup [OPTION]... \n\
  #             \n\
  #             Options: \n\
  #               --handshake                  Do connection/channel handshake for ping pong. \n\
  #               -g, --galois-url             Endpoint that serves galois. (Default: %s) \n\
  #               --galois-tls                 Connect to galois using TLS. \n\
  #               -c, --circuit-path           Path to the circuit files to run galois locally (if not specified, galois won't be run). \n\
  #               --evm-beacon-rpc-url         Rpc endpoint for the evm beacon chain. (Default: %s) \n\
  #               --evm-ws-url                 Websocket endpoint for the evm execution chain (Default: %s). \n\
  #               --union-rpc-url              Rpc endpoint for union (Default: %s). \n\
  #               --union-grpc-url             gRpc endpoint for union (Default: %s). \n\
  #               --union-ws-url               Websocket endpoint for union (Default: %s). \n\
  #               --voyager-config-file        Path to voyager config file. If not specified and --no-deploy-evm \n\
  #                                              is not given, a temp location will be used. If --no-deploy-evm is enabled, \n\
  #                                              this file is used as the voyager config. \n\
  #               --handshake                  Do an IBC handshake for ping pong. \n\
  #               --no-deploy-contracts        Don't deploy contracts, the voyager configuration file must be specified in this case. \n\
  #               --no-run-voyager             Don't run voyager for packet relaying, only print the command. \n\
  #               \n\
  #             Ping pong options:
  #               --ping-pong-address          Address of the ping pong app module on EVM. \n\
  #               --ping-pong-timeout          Number of blocks required for a pong message to timeout (Default: %s). \n\
  #               -h, --help                   Print help. \n\
  #             \n\
  #             Examples: \n\
  #               Use an already running galois: \n\
  #                 e2e-setup --galois-url http://some-server.com:16657 \n\
  #               Start a local galois: \n\
  #                 e2e-setup --circuit-path ./  \n\
  #               Use a custom voyager config and don't deploy the contracts: \n\
  #                 e2e-setup --voyager-config-file ~/.config/voyager/config.json --no-deploy-contracts \n\
  #               Do an IBC handshake for ping pong: \n\
  #                 e2e-setup --handshake \n\
  #               Don't run the voyager, only print the command to run it: \n\
  #                 e2e-setup --no-run-voyager
  #           " "$DEFAULT_GALOIS_URL" "$DEFAULT_EVM_BEACON_RPC_URL" "$DEFAULT_EVM_WS_URL" "$DEFAULT_UNION_RPC_URL" "$DEFAULT_UNION_RPC_URL" "$DEFAULT_UNION_WS_URL" "$DEFAULT_PING_PONG_TIMEOUT"
  #         }

  #         while [[ $# -gt 0 ]]; do
  #           case $1 in
  #             --handshake)
  #               HANDSHAKE=1
  #               shift
  #               ;;
  #             -g|--galois-url)
  #               GALOIS_URL="$2"
  #               shift
  #               shift
  #               ;;
  #             --galois-tls)
  #               GALOIS_TLS="--tls=1"
  #               shift
  #               ;;
  #             -c|--circuit-path)
  #               CIRCUIT_PATH="$2"
  #               shift
  #               shift
  #               ;;
  #             --evm-beacon-rpc-url)
  #               EVM_BEACON_RPC_URL="$2"
  #               shift
  #               shift
  #               ;;
  #             --evm-ws-url)
  #               EVM_WS_URL="$2"
  #               shift
  #               shift
  #               ;;
  #             --union-rpc-url)
  #               UNION_RPC_URL="$2"
  #               shift
  #               shift
  #               ;;
  #             --union-grpc-url)
  #               UNION_GRPC_URL="$2"
  #               shift
  #               shift
  #               ;;
  #             --union-ws-url)
  #               UNION_WS_URL="$2"
  #               shift
  #               shift
  #               ;;
  #             --voyager-config-file)
  #               VOYAGER_CONFIG_FILE="$2"
  #               shift
  #               shift
  #               ;;
  #             --union-dump-path)
  #               UNION_DUMP_PATH="$2"
  #               shift
  #               shift
  #               ;;
  #             --no-deploy-contracts)
  #               NO_DEPLOY_CONTRACTS=1
  #               shift
  #               ;;
  #             --ping-pong-address)
  #               PING_PONG_MODULE_ADDRESS="$2"
  #               shift
  #               shift
  #               ;;
  #             --ping-pong-timeout)
  #               PING_PONG_TIMEOUT="$2"
  #               shift
  #               shift
  #               ;;
  #             --no-run-voyager)
  #               NO_RUN_VOYAGER=1
  #               shift
  #               ;;
  #             -h|--help)
  #               printHelp
  #               exit 0
  #               ;;
  #             *)
  #               printHelp
  #               exit 1
  #               ;;
  #           esac
  #         done

  #         if [[ -n "$NO_DEPLOY_CONTRACTS" ]] && [[ -z "$PING_PONG_MODULE_ADDRESS" ]]; then
  #           echo "--ping-pong-address is required when --no-deploy-evm is enabled."
  #           printHelp
  #           exit 1
  #         fi

  #         if [[ -z "$VOYAGER_CONFIG_FILE" ]] && [[ -n "$NO_DEPLOY_CONTRACTS" ]]; then
  #           echo "--voyager-config-file must be specified when --no-deploy-evm is enabled."
  #           printHelp
  #           exit 1
  #         fi

  #         if [[ -z "$VOYAGER_CONFIG_FILE" ]]; then
  #           VOYAGER_CONFIG_FILE="$(mktemp -d)/voyager-config.json"
  #           echo "+ Created the voyager configuration file: $VOYAGER_CONFIG_FILE"
  #         fi

  #         if [[ -z "$UNION_DUMP_PATH" ]]; then
  #           UNION_DUMP_PATH="$(mktemp -d)"
  #         fi

  #         # Check if union devnet is running
  #         unionAliveTest() {
  #           curl -sS -X GET "$UNION_RPC_URL/status" -H 'Content-Type: application/json' 1>/dev/null
  #         }

  #         # Check if eth devnet is running
  #         ethAliveTest() {
  #           curl -sS -X GET "$EVM_BEACON_RPC_URL/eth/v1/beacon/blocks/finalized" -H 'accept: application/json' 1>/dev/null
  #         }

  #         # This downloads the circuit if its not up-to date or if it doesn't exist
  #         downloadGaloisCircuits() {
  #           if [[ -n "$CIRCUIT_PATH" ]]; then
  #               echo "Checking if we need to download the circuit.."
  #               ${self'.packages.download-circuit}/bin/download-circuit devnet "$CIRCUIT_PATH"
  #           else
  #             echo "Skipping downloading the circuit since --galois-url is provided."
  #           fi
  #         }

  #         runGalois() {
  #           echo Checking if circuits exists at "$CIRCUIT_PATH"..
  #           if [ ! -f "$CIRCUIT_PATH/r1cs.bin" ] || [ ! -f "$CIRCUIT_PATH/pk.bin" ] || [ ! -f "$CIRCUIT_PATH/vk.bin" ]; then
  #             echo "Some files are still missing. Please re-run the command to download the files.."
  #             exit 1
  #           fi
  #           echo "Starting galois.."
  #           ${self'.packages.galoisd-devnet}/bin/galoisd serve "$GALOIS_URL"
  #         }

  #         EVM_CONTRACTS_OUTFILE=$(mktemp)

  #         deployEVMPingPong() {
  #           export IBC_HANDLER_ADDRESS
  #           export NUM_OF_BLOCK_BEFORE_PONG_TIMEOUT="$PING_PONG_TIMEOUT"
  #           export REVISION_NUMBER=1

  #           echo ------------------------------------
  #           echo + Deploying Ping Pong App..
  #           ${self'.packages.evm-devnet-ping-pong-deploy}/bin/evm-devnet-ping-pong-deploy | tee "$EVM_CONTRACTS_OUTFILE"
  #           echo ------------------------------------
  #           EVM_CONTRACTS_ARG=$(tail -1 < "$EVM_CONTRACTS_OUTFILE")
  #           rm "$EVM_CONTRACTS_OUTFILE"

  #           PING_PONG_MODULE_ADDRESS=$(echo "$EVM_CONTRACTS_ARG" | jq .ping_pong_address -r)
  #         }

  #         deployEVMContracts() {
  #           echo ------------------------------------
  #           echo + Deploying IBC Contracts..
  #           while ! ${self'.packages.evm-devnet-deploy}/bin/evm-devnet-deploy | tee "$EVM_CONTRACTS_OUTFILE"
  #           do
  #             echo "Eth doesn't seem to be ready yet. Will try in 3 seconds."
  #             sleep 3
  #           done
  #           echo ------------------------------------

  #           EVM_CONTRACTS_ARG=$(tail -1 < "$EVM_CONTRACTS_OUTFILE")
  #           rm "$EVM_CONTRACTS_OUTFILE"

  #           COMETBLS_ADDRESS=$(echo "$EVM_CONTRACTS_ARG" | jq .cometbls_client_address -r)
  #           IBC_HANDLER_ADDRESS=$(echo "$EVM_CONTRACTS_ARG" | jq .ibc_handler_address -r)
  #           UCS01RELAY=$(echo "$EVM_CONTRACTS_ARG" | jq .ucs01_relay_address -r)
  #           WASM_CODE_ID=$(cat ${self'.packages.devnet-genesis}/code-ids/ethereum_light_client_minimal)
  #           EVM_WALLET=$(cat ${self'.packages.devnet-evm-config}/dev-key0.prv)

  #           echo '{
  #             "chain": {
  #               "ethereum-devnet": {
  #                 "chain_type": "evm",
  #                 "preset_base": "minimal",
  #                 "cometbls_client_address": "'"$COMETBLS_ADDRESS"'",
  #                 "ibc_handler_address": "'"$IBC_HANDLER_ADDRESS"'",
  #                 "signer": {
  #                   "raw": "0x'"$EVM_WALLET"'"
  #                 },
  #                 "eth_rpc_api": "'"$EVM_WS_URL"'",
  #                 "eth_beacon_rpc_api": "'"$EVM_BEACON_RPC_URL"'",
  #                 "wasm_code_id": "0x'"$WASM_CODE_ID"'"
  #               },
  #               "union-devnet": {
  #                 "chain_type": "union",
  #                 "signer": {
  #                   "raw": "0xaa820fa947beb242032a41b6dc9a8b9c37d8f5fbcda0966b1ec80335b10a7d6f"
  #                 },
  #                 "ws_url": "'"$UNION_WS_URL"'",
  #                 "wasm_code_id": "0x'"$WASM_CODE_ID"'",
  #                 "prover_endpoint": "'"$GALOIS_URL"'",
  #                 "dump_path": "'"$UNION_DUMP_PATH"'",
  #                 "grpc_url": "'"$UNION_GRPC_URL"'"
  #               }
  #             }              }' | jq > "$VOYAGER_CONFIG_FILE"

  #             deployEVMPingPong
  #         }

  #         instantiateCwUCS01() {
  #           ${instantiateCwUCS01}/bin/instantiate-ucs01-relay
  #         }

  #         instantiatePingPong() {
  #           ${instantiatePingPong}/bin/instantiate-ping-pong "$PING_PONG_TIMEOUT"
  #         }

  #         createClients() {
  #           echo ------------------------------------
  #           echo "+ Creating light client on evm."
  #           RUST_LOG=voyager=debug ${self'.packages.voyager}/bin/voyager \
  #             --config-file-path "$VOYAGER_CONFIG_FILE" \
  #             client create evm cometbls \
  #             --cometbls-client-address "$COMETBLS_ADDRESS" \
  #             --on ethereum-devnet \
  #             --counterparty union-devnet
  #           echo ------------------------------------
  #           echo "+ Creating client on union."
  #           RUST_LOG=voyager=debug ${self'.packages.voyager}/bin/voyager \
  #             --config-file-path "$VOYAGER_CONFIG_FILE" \
  #             client create union ethereum08-wasm \
  #             --on union-devnet \
  #             --counterparty ethereum-devnet \
  #             --code-id "0x$WASM_CODE_ID" \
  #             --evm-preset minimal
  #           echo ------------------------------------
  #         }

  #         waitForGaloisToBeOnline() {
  #           COMMAND="${self'.packages.galoisd-devnet}/bin/galoisd gen-contract $GALOIS_URL"

  #           if [[ -n "$GALOIS_TLS" ]]; then
  #             COMMAND="$COMMAND $GALOIS_TLS"
  #           fi

  #           while ! eval "$COMMAND" 1>/dev/null 2>&1
  #           do
  #             echo ".. Waiting for galois to be ready at $GALOIS_URL .."
  #             sleep 5
  #           done
  #         }

  #         setupInitialChannel() {
  #           MODULE_ADDRESS=$1
  #           PORT_ID=$2
  #           COUNTERPARTY_PORT_ID=$3
  #           CHANNEL_ID=$4

  #           echo ------------------------------------------------------------
  #           echo "+ Setting up the initial connection and channels on EVM.."
  #           ${self'.packages.voyager}/bin/voyager \
  #             --config-file-path "$VOYAGER_CONFIG_FILE" \
  #             setup initial-channel \
  #             --on ethereum-devnet \
  #             --module-address "$MODULE_ADDRESS" \
  #             --channel-id "$CHANNEL_ID" \
  #             --port-id "$PORT_ID" \
  #             --counterparty-port-id "$COUNTERPARTY_PORT_ID"
  #           echo "+ Initial connection and channels are ready."
  #         }

  #         doHandshake() {
  #             from_version="$1"
  #             to_version="$2"

  #             echo ------------------------------------------------------------
  #             echo "+ Doing connection and channel handshakes.."
  #             echo RUST_LOG=voyager=debug ${self'.packages.voyager}/bin/voyager \
  #             --config-file-path "$VOYAGER_CONFIG_FILE" \
  #               connection open \
  #               --to-chain union-devnet \
  #               --to-client 08-wasm-0 \
  #               --from-chain ethereum-devnet \
  #               --from-client cometbls-new-0

  #             RUST_LOG=voyager=debug ${self'.packages.voyager}/bin/voyager \
  #             --config-file-path "$VOYAGER_CONFIG_FILE" \
  #               connection open \
  #               --to-chain union-devnet \
  #               --to-client 08-wasm-0 \
  #               --from-chain ethereum-devnet \
  #               --from-client cometbls-new-0

  #             RUST_LOG=voyager=debug ${self'.packages.voyager}/bin/voyager \
  #               --config-file-path "$VOYAGER_CONFIG_FILE" \
  #               setup bind-port \
  #               --on ethereum-devnet \
  #               --module-address "$PING_PONG_MODULE_ADDRESS" \
  #               --port-id "ping-pong"

  #             RUST_LOG=voyager=debug ${self'.packages.voyager}/bin/voyager \
  #               --config-file-path "$VOYAGER_CONFIG_FILE" \
  #               channel open \
  #               --to-chain union-devnet \
  #               --to-connection connection-1 \
  #               --to-port "wasm.$PING_PONG_ADDRESS" \
  #               --to-version "$to_version" \
  #               --from-chain ethereum-devnet \
  #               --from-connection connection-1 \
  #               --from-port "ping-pong" \
  #               --from-version "$from_version"
  #         }

  #         printIBCSetupInfo() {
  #           echo ---------------------------------------------------
  #           echo "+ Module $1(EVM) and $2(Union) is connected at:"
  #           echo "    - EVM:"
  #           echo "      - Address:     $3"
  #           echo "      - Connection:  $4"
  #           echo "      - Channel:     $5"
  #           echo "      - Port:        $6"
  #           echo "    - Union:"
  #           echo "      - Address:     $7"
  #           echo "      - Connection:  $8"
  #           echo "      - Channel:     $9"

  #           shift
  #           echo "      - Port:        $9"
  #         }

  #         downloadGaloisCircuits
  #         ethAliveTest
  #         unionAliveTest

  #         if [[ -z "$NO_DEPLOY_CONTRACTS" ]]; then
  #           instantiateCwUCS01
  #           sleep 6
  #           instantiatePingPong
  #           deployEVMContracts
  #         else
  #           echo "--no-deploy-contracts is specified, assuming contracts on both sides are ready."
  #         fi

  #         # Voyager requires the scheme to be included (http(s)) but galoisd returns an error when
  #         # it is run with a scheme in the URL.
  #         # TODO(aeryz): This should not be the case, this should probably be fixed in galois
  #         GALOIS_URL=$(echo "$GALOIS_URL" | sed -e "s/^http:\/\///" | sed -e "s/^https:\/\///")

  #         if [[ -n "$CIRCUIT_PATH" ]]; then
  #           runGalois &
  #         else
  #           echo "+ --circuit-path is empty, will use the galois at $GALOIS_URL"
  #         fi

  #         CW20_ADDRESS=$(${uniond} query wasm list-contract-by-code 1  --output json | jq '.contracts[0]' -r)
  #         PING_PONG_ADDRESS=$(${uniond} query wasm list-contract-by-code 2  --output json | jq '.contracts[0]' -r)

  #         if [[ -z "$CW20_ADDRESS" ]] || [[ -z "$PING_PONG_ADDRESS" ]]; then
  #           echo "Couldn't find the uploaded contract. Something is wrong. Please start over."
  #           exit 0
  #         fi;

  #         setupInitialChannel "$UCS01RELAY" transfer "wasm.$CW20_ADDRESS" channel-0

  #         if [[ -z "$HANDSHAKE" ]]; then
  #           PING_PONG_CONNECTION="connection-0"
  #           PING_PONG_CHANNEL="channel-1"
  #           setupInitialChannel "$PING_PONG_MODULE_ADDRESS" ping-pong "wasm.$PING_PONG_ADDRESS" channel-1
  #         fi

  #         createClients
  #         waitForGaloisToBeOnline

  #         if [[ -n "$HANDSHAKE" ]]; then
  #           # Since we already embedded connection-0 to the genesis, if we manually do the handshake,
  #           # the connection id is going to be connection-1
  #           PING_PONG_CONNECTION="connection-1"
  #           PING_PONG_CHANNEL="channel-2"
  #           doHandshake "ucs00-pingpong-1" "ucs00-pingpong-1"
  #         fi

  #         # We need the home so that the users can send transactions by using the testkey
  #         TX_HOME="$(mktemp -d)"
  #         cp -r ${self'.packages.devnet-genesis}/* "$TX_HOME"

  #         echo "--------------------------------"
  #         echo "+ Voyager config path is: $VOYAGER_CONFIG_FILE"
  #         echo "+ The home path that you can use for union transactions is: $TX_HOME"

  #         printIBCSetupInfo \
  #           "UCS01 Transfer" \
  #           "UCS01-RELAY" \
  #           "$UCS01RELAY" \
  #           "connection-0" \
  #           "channel-0" \
  #           "transfer" \
  #           "$CW20_ADDRESS" \
  #           "connection-0" \
  #           "channel-0" \
  #           "wasm.$CW20_ADDRESS"

  #         echo "---------------------------------------------------------------------"
  #         echo "+ To run this app on union, run the following command:"
  #         echo '${uniond}  \
  #           tx wasm execute '"$CW20_ADDRESS"' \
  #           "{\"transfer\":{\"channel\":\"channel-0\",\"remote_address\":\"be68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed\"}}" \
  #           --gas-adjustment 1.3 \
  #           --gas auto \
  #           --from ${keyName} -y \
  #           --keyring-backend test \
  #           --chain-id ${chainId} \
  #           --home '"$TX_HOME"' \
  #           --amount 123123stake'

  #         echo "---------------------------------------------------------------------"
  #         echo "+ To send some funds back to union from ethereum, make sure the packet is relayed first and then run the following:"
  #         echo 'RUST_LOG=voyager=debug ${self'.packages.voyager}/bin/voyager \
  #           --config-file-path '"$VOYAGER_CONFIG_FILE"' \
  #           submit-packet transfer \
  #           --on ethereum-devnet \
  #           --denom transfer/channel-0/stake \
  #           --amount 10000 \
  #           --source-port transfer \
  #           --source-channel channel-0 \
  #           --receiver '"$(${accountAddress})"

  #         printIBCSetupInfo \
  #           "PingPong" \
  #           "PingPong" \
  #           "$PING_PONG_MODULE_ADDRESS" \
  #           "$PING_PONG_CONNECTION" \
  #           "channel-1" \
  #           "ping-pong" \
  #           "$PING_PONG_ADDRESS" \
  #           "$PING_PONG_CONNECTION" \
  #           "$PING_PONG_CHANNEL" \
  #           "wasm.$PING_PONG_ADDRESS"

  #         echo "---------------------------------------------------------------------"
  #         echo "+ To start the ping pong, run the following command:"
  #         echo '${uniond}  \
  #           tx wasm execute '"$PING_PONG_ADDRESS"' \
  #           "{\"initiate\":{\"channel_id\":\"'"$PING_PONG_CHANNEL"'\",\"packet\":{\"ping\":true,\"counterparty_timeout_revision_number\":0,\"counterparty_timeout_revision_height\":1000000}}}" \
  #           --gas-adjustment 1.3 \
  #           --gas auto \
  #           --from ${keyName} -y \
  #           --keyring-backend test \
  #           --chain-id ${chainId} \
  #           --home '"$TX_HOME"

  #         echo "----------------------------------------------------------------"

  #         VOYAGER_CMD='RUST_LOG=voyager=debug ${self'.packages.voyager}/bin/voyager \
  #           --config-file-path '"$VOYAGER_CONFIG_FILE"' \
  #           relay \
  #           --between union-devnet:ethereum-devnet'

  #         if [[ -z "$NO_RUN_VOYAGER" ]]; then
  #           echo "+ Starting the voyager to relay the packets.."
  #           eval "$VOYAGER_CMD"
  #         else
  #           echo "+ Run voyager to relay the packets with the following command:"
  #           echo "$VOYAGER_CMD"
  #         fi

  #         wait
  #       '';
  #     };
  # };
}
