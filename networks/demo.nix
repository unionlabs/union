{ ... }: {
  perSystem = { devnetConfig, pkgs, self', inputs', ... }:
    let
    uniond = pkgs.lib.getExe self'.packages.uniond;

    # Name of the account key that we use for the devnet
    keyName = "testkey";
    chainId = "union-devnet-1";
    # This can be any arbitrary hex string which is used when generating contract address
    salt = "61616161";

    instantiate-cw20-ics20 =
      pkgs.writeShellApplication {
        name = "instantiate-cw20-ics20";
        runtimeInputs = [];
        text =
      ''
        # This account will be the governor and admin of the contract that we instantiate
        ACCOUNT_ADDRESS=$(${uniond} keys show ${keyName} \
          --keyring-backend test \
          --address \
          --home ${self'.packages.devnet-genesis})

        # TODO(aeryz): Read the code_id from genesis setup
        # Instantiate cw20 contract with an initial channel. The contract will automatically assign
        # the correct port id to the channel.
        while ! ${uniond} tx wasm instantiate2 1 \
          '{
            "default_timeout":300,
            "gov_contract": "'"$ACCOUNT_ADDRESS"'",
            "allowlist":[],
            "channel":{
              "endpoint":{
                "port_id": "",
                "channel_id":"channel-0"
              },
              "counterparty_endpoint":{
                "port_id":"transfer",
                "channel_id":"channel-0"
              },
              "order":"ORDER_UNORDERED",
              "version":"ics20-1",
              "connection_id":"connection-0"
            }
          }' \
          ${salt} \
          --label cw20-ics20-test \
          --gas=auto \
          --gas-adjustment=1.3 -y  \
          --admin "$ACCOUNT_ADDRESS" \
          --keyring-backend test \
          --from ${keyName} \
          --chain-id ${chainId} \
          --home ${self'.packages.devnet-genesis}
        do
          sleep 1
        done
      '';
      };

    instantiate-ping-pong =
      pkgs.writeShellApplication {
        name = "instantiate-ping-pong";
        runtimeInputs = [];
        text =
      ''
        # This account will be the governor and admin of the contract that we instantiate
        ACCOUNT_ADDRESS=$(${uniond} keys show ${keyName} \
          --keyring-backend test \
          --address \
          --home ${self'.packages.devnet-genesis})

        # TODO(aeryz): Read the code_id from genesis setup
        # Instantiate cw20 contract with an initial channel. The contract will automatically assign
        # the correct port id to the channel.
        while ! ${uniond} tx wasm instantiate2 2 \
          '{}' \
          ${salt} \
          --label ping-pong-test \
          --gas=auto \
          --gas-adjustment=1.3 -y  \
          --admin "$ACCOUNT_ADDRESS" \
          --keyring-backend test \
          --from ${keyName} \
          --chain-id ${chainId} \
          --home ${self'.packages.devnet-genesis}
        do
          sleep 1
        done
      '';
      };

    in
    {
      packages.setup-demo =
        pkgs.writeShellApplication {
          name = "union-devnet-demo";
          runtimeInputs = [ pkgs.jq ];
          text = ''
            # Kill all subprocesses the root process dies
            trap "kill 0" EXIT

            GALOIS_URL="http://0.0.0.0:16657"
            CIRCUIT_PATH=""
            EVM_BEACON_RPC_URL="http://localhost:9596"
            EVM_WS_URL="ws://localhost:8546"
            UNION_RPC_URL="http://localhost:26657"
            UNION_WS_URL="ws://localhost:26657"
            RELAYER_CONFIG_FILE=""
            NO_DEPLOY_EVM=""
            PING_PONG_MODULE_ADDRESS=""

            printHelp() {
              printf " \
                Usage: nix run .#setup-demo [OPTION]... \n\
                \n\
                Options: \n\
                  -g, --galois-url        Endpoint that serves galois (Default: http://0.0.0.0:16657) \n\
                  -c, --circuit-path      Path to the circuit files to run galois locally (if not specified, galois won't be run) \n\
                  --evm-beacon-rpc-url    Rpc endpoint for the evm beacon chain (Default: http://localhost:9596) \n\
                  --evm-ws-url            Websocket endpoint for the evm execution chain (Default: ws://localhost:8546) \n\
                  --union-rpc-url         Rpc endpoint for union (Default: http://localhost:26657) \n\
                  --union-ws-url          Websocket endpoint for union (Default: ws://localhost:26657/websocket) \n\
                  --relayer-config-file   Path to relayer config file. If not specified and --no-deploy-evm \n\
                  \t is not given, a temp location will be used. If --no-deploy-evm is enabled, this file is used as the relayer config \n\
                  --no-deploy-evm         Don't deploy evm contracts \n\
                  --ping-pong-address     Address of the ping pong app module on EVM \n\
                  -h, --help            Print help \n\
                \n\
                Examples: \n\
                  Use an already running galois: \n\
                    nix run .#setup-demo -- --galois-url http://some-server.com:16657 \n\
                  Start a local galois: \n\
                    nix run .#setup-demo -- --circuit-path ./  \n\
              "
            }

            while [[ $# -gt 0 ]]; do
              case $1 in
                -g|--galois-url)
                  GALOIS_URL="$2"
                  shift
                  shift
                  ;;
                -c|--circuit-path)
                  CIRCUIT_PATH="$2"
                  shift
                  shift
                  ;;
                --evm-beacon-rpc-url)
                  EVM_BEACON_RPC_URL="$2"
                  shift
                  shift
                  ;;
                --evm-ws-url)
                  EVM_WS_URL="$2"
                  shift
                  shift
                  ;;
                --union-rpc-url)
                  UNION_RPC_URL="$2"
                  shift
                  shift
                  ;;
                --union-ws-url)
                  UNION_WS_URL="$2"
                  shift
                  shift
                  ;;
                --relayer-config-file)
                  RELAYER_CONFIG_FILE="$2"
                  shift
                  shift
                  ;;
                --no-deploy-evm)
                  NO_DEPLOY_EVM=1
                  shift
                  ;;
                --ping-pong-address)
                  PING_PONG_MODULE_ADDRESS="$2"
                  shift
                  shift
                  ;;
                -h|--help)
                  printHelp
                  exit 0
                  ;;
              esac
            done

            if [[ -n "$NO_DEPLOY_EVM" ]] && [[ -z "$PING_PONG_MODULE_ADDRESS" ]]; then
              echo "--ping-pong-address is required when --no-deploy-evm is enabled."
              printHelp
              exit 1
            fi
            

            if [[ -z "$RELAYER_CONFIG_FILE" ]] && [[ -n "$NO_DEPLOY_EVM" ]]; then
              echo "--relayer-config-file must be specified when --no-deploy-evm is enabled."
              printHelp
              exit 1
            fi

            if [[ -z "$RELAYER_CONFIG_FILE" ]]; then
              RELAYER_CONFIG_FILE="$(mktemp -d)/relayer-config.json"
            fi

            # Check if union devnet is running
            unionAliveTest() {
              curl -sS -X GET "$UNION_RPC_URL/status" -H 'Content-Type: application/json' 1>/dev/null
            }

            # Check if eth devnet is running
            ethAliveTest() {
              curl -sS -X GET "$EVM_BEACON_RPC_URL/eth/v1/beacon/blocks/finalized" -H 'accept: application/json' 1>/dev/null
            }

            # This downloads the circuit if its not up-to date or if it doesn't exist
            downloadGaloisCircuits() {
              if [[ -n "$CIRCUIT_PATH" ]]; then
                  echo "Checking if we need to download the circuit.."
                  ${self'.packages.download-circuit}/bin/download-circuit devnet "$CIRCUIT_PATH"
              else
                echo "Skipping downloading the circuit since --galois-url is provided."
              fi
            }

            runGalois() {
              echo Checking if circuits exists at "$CIRCUIT_PATH"..
              if [ ! -f "$CIRCUIT_PATH/r1cs.bin" ] || [ ! -f "$CIRCUIT_PATH/pk.bin" ] || [ ! -f "$CIRCUIT_PATH/vk.bin" ]; then
                echo "Some files are still missing. Please re-run the command to download the files.."
                exit 1
              fi                  
              echo "Starting galois.."
              ${self'.packages.galoisd-devnet}/bin/galoisd serve "$GALOIS_URL"
            }

            EVM_CONTRACTS_OUTFILE=$(mktemp)

            deployEVMContracts() {
              while ! ${self'.packages.evm-devnet-deploy}/bin/evm-devnet-deploy | tee "$EVM_CONTRACTS_OUTFILE" 
              do
                sleep 1
              done

              EVM_CONTRACTS_ARG=$(tail -1 < "$EVM_CONTRACTS_OUTFILE")
              rm "$EVM_CONTRACTS_OUTFILE"


              COMETBLS_ADDRESS=$(echo "$EVM_CONTRACTS_ARG" | jq .cometbls_client_address -r)
              IBC_HANDLER_ADDRESS=$(echo "$EVM_CONTRACTS_ARG" | jq .ibc_handler_address -r)
              ICS20_TRANSFER_BANK_ADDRESS=$(echo "$EVM_CONTRACTS_ARG" | jq .ics20_transfer_bank_address -r)
              ICS20_BANK_ADDRESS=$(echo "$EVM_CONTRACTS_ARG" | jq .ics20_bank_address -r)
              PING_PONG_MODULE_ADDRESS=$(echo "$EVM_CONTRACTS_ARG" | jq .ping_pong_address -r)
              WASM_CODE_ID=$(cat ${self'.packages.devnet-genesis}/code-ids/ethereum_light_client_minimal)
              EVM_WALLET=$(cat ${self'.packages.devnet-evm-config}/dev-key0.prv)

              echo '{
                "chain": {
                  "ethereum-devnet": {
                    "chain_type": "evm",
                    "preset_base": "minimal",
                    "cometbls_client_address": "'"$COMETBLS_ADDRESS"'",
                    "ibc_handler_address": "'"$IBC_HANDLER_ADDRESS"'",
                    "ics20_transfer_bank_address": "'"$ICS20_TRANSFER_BANK_ADDRESS"'",
                    "ics20_bank_address": "'"$ICS20_BANK_ADDRESS"'",
                    "signer": {
                      "raw": "0x'"$EVM_WALLET"'"
                    },
                    "eth_rpc_api": "'"$EVM_WS_URL"'",
                    "eth_beacon_rpc_api": "'"$EVM_BEACON_RPC_URL"'",
                    "wasm_code_id": "0x'"$WASM_CODE_ID"'"
                  },
                  "union-devnet": {
                    "chain_type": "union",
                    "signer": {
                      "raw": "0xaa820fa947beb242032a41b6dc9a8b9c37d8f5fbcda0966b1ec80335b10a7d6f"
                    },
                    "ws_url": "'"$UNION_WS_URL"'",
                    "wasm_code_id": "0x'"$WASM_CODE_ID"'",
                    "prover_endpoint": "'"$GALOIS_URL"'"
                  }
                }
              }' | jq > "$RELAYER_CONFIG_FILE"

            }

            instantiateCw20Ics20() {
              ${instantiate-cw20-ics20}/bin/instantiate-cw20-ics20
            }

            instantiatePingPong() {
              ${instantiate-ping-pong}/bin/instantiate-ping-pong
            }

            createClients() {
              echo "Creating light client on evm."
              RUST_LOG=relayer=info ${self'.packages.relayer}/bin/relayer \
                --config-file-path "$RELAYER_CONFIG_FILE" \
                client create evm cometbls \
                --on ethereum-devnet \
                --counterparty union-devnet

              echo "Creating client on union."
              RUST_LOG=relayer=info ${self'.packages.relayer}/bin/relayer \
                --config-file-path "$RELAYER_CONFIG_FILE" \
                client create union ethereum08-wasm \
                --on union-devnet \
                --counterparty ethereum-devnet \
                --evm-preset minimal
            }

            startRelaying() {
              while ! ${self'.packages.galoisd-devnet}/bin/galoisd query-stats "$GALOIS_URL"
              do 
                echo "Waiting for galois to be ready at $GALOIS_URL"
                sleep 2 
              done

              echo "Starting the relayer.."
              echo "+ Relayer config path is: $RELAYER_CONFIG_FILE"
              echo "+ cw20-ics20 port id is: $COUNTERPARTY_PORT_ID"
              echo "+ Channel and connection ids on both chains are: channel-0 and connection-0"

              RUST_LOG=relayer=info ${self'.packages.relayer}/bin/relayer \
                --config-file-path "$RELAYER_CONFIG_FILE" \
                relay \
                --between union-devnet:ethereum-devnet
            }

            setupInitialChannel() {
              MODULE_ADDRESS=$1
              PORT_ID=$2
              COUNTERPARTY_PORT_ID=$3
              CHANNEL_ID=$4

              echo "Setting up the initial connection and channels on EVM.."
              ${self'.packages.relayer}/bin/relayer \
                --config-file-path "$RELAYER_CONFIG_FILE" \
                setup initial-channel \
                --on ethereum-devnet \
                --module-address "$MODULE_ADDRESS" \
                --channel-id = "$CHANNEL_ID" \
                --port-id "$PORT_ID" \
                --counterparty-port-id "$COUNTERPARTY_PORT_ID"

            }

            downloadGaloisCircuits
            ethAliveTest
            unionAliveTest
            instantiateCw20Ics20
            sleep 6
            instantiatePingPong

            if [[ -z "$NO_DEPLOY_EVM" ]]; then
              deployEVMContracts
            else
              echo "Won't be deploying the evm contracts since --no-deploy-evm is on."
            fi

            if [[ -n "$CIRCUIT_PATH" ]]; then
              runGalois &
            else
              echo "--circuit-path is empty, will use the galois at $GALOIS_URL"
            fi

            ICS20_TRANSFER_BANK_ADDRESS=$(jq '.chain."ethereum-devnet".ics20_transfer_bank_address' -r < "$RELAYER_CONFIG_FILE")
            CW20_ADDRESS=$(${uniond} query wasm list-contract-by-code 1  --output json | jq '.contracts[0]' -r)
            PING_PONG_ADDRESS=$(${uniond} query wasm list-contract-by-code 2  --output json | jq '.contracts[0]' -r)

            if [[ -z "$CW20_ADDRESS" ]] || [[ -z "$PING_PONG_ADDRESS" ]]; then
              echo "Couldn't find the uploaded contract. Something is wrong. Please start over."
              exit 0
            fi;

            setupInitialChannel "$ICS20_TRANSFER_BANK_ADDRESS" transfer "wasm.$CW20_ADDRESS" channel-0
            setupInitialChannel "$PING_PONG_MODULE_ADDRESS" ping-pong "wasm.$PING_PONG_ADDRESS" channel-1

            createClients
            startRelaying

            wait
          '';
        };
    };
}
