{ ... }: {
  perSystem = { e2e, devnetConfig, pkgs, self', inputs', cw-instantiate2-salt, ... }:
    let
      uniond = pkgs.lib.getExe self'.packages.uniond;

      # Name of the account key that we use for the devnet
      keyName = "testkey";
      chainId = "union-devnet-1";

      accountAddress = ''
        ${uniond} keys show ${keyName} \
          --keyring-backend test \
          --address \
          --home ${self'.packages.devnet-genesis}
      '';

      instantiateContract = { code-id, label }:
        ''
          ACCOUNT_ADDRESS="$(${accountAddress})"

          echo ------------------------------------
          echo + Instantiating ${label}:
          echo     - Message: "$(echo "$INIT_MESSAGE" | jq)"
          echo ------------------------------------

          while ! ${uniond} tx wasm instantiate2 ${toString code-id} \
            "$INIT_MESSAGE" \
            ${cw-instantiate2-salt} \
            --label ${label} \
            --gas=auto \
            --gas-adjustment=1.3 -y  \
            --admin "$ACCOUNT_ADDRESS" \
            --keyring-backend test \
            --from ${keyName} \
            --chain-id ${chainId} \
            --home ${self'.packages.devnet-genesis} > /dev/null
          do
            echo "Chain doesn't seem to be ready yet. Will retry in 3 seconds."
            sleep 3
          done
        '';

      instantiateCw20Ics20 =
        pkgs.writeShellApplication {
          name = "instantiate-cw20-ics20";
          runtimeInputs = [ ];
          text =
            ''
              # This account will be the governor and admin of the contract that we instantiate
              ACCOUNT_ADDRESS="$(${accountAddress})"

              INIT_MESSAGE='{
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
                }'

              ${instantiateContract { code-id = 1; label = "cw20-ics20"; }}
            '';
        };

      instantiatePingPong =
        pkgs.writeShellApplication {
          name = "instantiate-ping-pong";
          runtimeInputs = [ ];
          text =
            ''
              TIMEOUT=$1
              INIT_MESSAGE='{
                  "config": {
                    "number_of_block_before_pong_timeout": '"$TIMEOUT"',
                    "revision_number": 1
                    }
                }'

              ${instantiateContract { code-id = 2; label = "ping-pong"; }}
            '';
        };
    in
    rec {
      checks.demo = e2e.mkTestWithDevnetSetup {
        name = "demo";

        testScript = ''
            client.succeed("${packages.setup-demo}/bin/union-devnet-demo -- --circuit-path ./")
        '';

        nodes = {
          # empty node used to communicate with the other nodes
          client = _: { };
        };
      };
      packages.setup-demo =
        pkgs.writeShellApplication {
          name = "union-devnet-demo";
          runtimeInputs = [ pkgs.jq ];
          text = ''
            # Kill all subprocesses the root process dies
            trap "kill 0" EXIT

            DEFAULT_GALOIS_URL="http://0.0.0.0:16657"
            DEFAULT_EVM_BEACON_RPC_URL="http://localhost:9596"
            DEFAULT_EVM_WS_URL="ws://localhost:8546"
            DEFAULT_UNION_RPC_URL="http://localhost:26657"
            DEFAULT_UNION_WS_URL="ws://localhost:26657/websocket"
            DEFAULT_PING_PONG_TIMEOUT=1000

            GALOIS_URL="$DEFAULT_GALOIS_URL"
            GALOIS_TLS=""
            CIRCUIT_PATH=""
            EVM_BEACON_RPC_URL="$DEFAULT_EVM_BEACON_RPC_URL"
            EVM_WS_URL="$DEFAULT_EVM_WS_URL"
            UNION_RPC_URL="$DEFAULT_UNION_RPC_URL"
            UNION_WS_URL="$DEFAULT_UNION_WS_URL"
            RELAYER_CONFIG_FILE=""
            UNION_DUMP_PATH=""
            NO_DEPLOY_EVM=""
            PING_PONG_MODULE_ADDRESS=""
            PING_PONG_TIMEOUT="$DEFAULT_PING_PONG_TIMEOUT"

            printHelp() {
              printf " \
                Usage: nix run .#setup-demo [OPTION]... \n\
                \n\
                Options: \n\
                  -g, --galois-url             Endpoint that serves galois. (Default: %s) \n\
                  --galois-tls                 Connect to galois using TLS. \n\
                  -c, --circuit-path           Path to the circuit files to run galois locally (if not specified, galois won't be run). \n\
                  --evm-beacon-rpc-url         Rpc endpoint for the evm beacon chain. (Default: %s) \n\
                  --evm-ws-url                 Websocket endpoint for the evm execution chain (Default: %s). \n\
                  --union-rpc-url              Rpc endpoint for union (Default: %s). \n\
                  --union-ws-url               Websocket endpoint for union (Default: %s). \n\
                  --relayer-config-file        Path to relayer config file. If not specified and --no-deploy-evm \n\
                                                 is not given, a temp location will be used. If --no-deploy-evm is enabled, \n\
                                                 this file is used as the relayer config. \n\
                  --no-deploy-evm              Don't deploy evm contracts. \n\
                  \n\
                Ping pong options:
                  --ping-pong-address          Address of the ping pong app module on EVM. \n\
                  --ping-pong-timeout          Number of blocks required for a pong message to timeout (Default: %s). \n\
                  -h, --help                   Print help. \n\
                \n\
                Examples: \n\
                  Use an already running galois: \n\
                    nix run .#setup-demo -- --galois-url http://some-server.com:16657 \n\
                  Start a local galois: \n\
                    nix run .#setup-demo -- --circuit-path ./  \n\
                  Use a custom relayer config and don't deploy evm contracts: \n\
                    nix run .#setup-demo -- --relayer-config-file ~/.config/relayer/config.json --no-deploy-evm
              " "$DEFAULT_GALOIS_URL" "$DEFAULT_EVM_BEACON_RPC_URL" "$DEFAULT_EVM_WS_URL" "$DEFAULT_UNION_RPC_URL" "$DEFAULT_UNION_WS_URL" "$DEFAULT_PING_PONG_TIMEOUT"
            }

            while [[ $# -gt 0 ]]; do
              case $1 in
                -g|--galois-url)
                  GALOIS_URL="$2"
                  shift
                  shift
                  ;;
                --galois-tls)
                  GALOIS_TLS="--tls=1"
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
                --union-dump-path)
                  UNION_DUMP_PATH="$2"
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
                --ping-pong-timeout)
                  PING_PONG_TIMEOUT="$2"
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

            if [[ -z "$UNION_DUMP_PATH" ]]; then
              UNION_DUMP_PATH="$(mktemp -d)"
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

            deployEVMPingPong() {
              export IBC_HANDLER_ADDRESS
              export NUM_OF_BLOCK_BEFORE_PONG_TIMEOUT="$PING_PONG_TIMEOUT"
              export REVISION_NUMBER=1              
              
              echo ------------------------------------
              echo + Deploying Ping Pong App..
              ${self'.packages.evm-devnet-ping-pong-deploy}/bin/evm-devnet-ping-pong-deploy | tee "$EVM_CONTRACTS_OUTFILE"
              echo ------------------------------------
              EVM_CONTRACTS_ARG=$(tail -1 < "$EVM_CONTRACTS_OUTFILE")
              rm "$EVM_CONTRACTS_OUTFILE"

              PING_PONG_MODULE_ADDRESS=$(echo "$EVM_CONTRACTS_ARG" | jq .ping_pong_address -r)
            }

            deployEVMContracts() {
              echo ------------------------------------
              echo + Deploying IBC Contracts..
              while ! ${self'.packages.evm-devnet-deploy}/bin/evm-devnet-deploy | tee "$EVM_CONTRACTS_OUTFILE"
              do
                echo "Eth doesn't seem to be ready yet. Will try in 3 seconds."
                sleep 3
              done
              echo ------------------------------------

              EVM_CONTRACTS_ARG=$(tail -1 < "$EVM_CONTRACTS_OUTFILE")
              rm "$EVM_CONTRACTS_OUTFILE"


              COMETBLS_ADDRESS=$(echo "$EVM_CONTRACTS_ARG" | jq .cometbls_client_address -r)
              IBC_HANDLER_ADDRESS=$(echo "$EVM_CONTRACTS_ARG" | jq .ibc_handler_address -r)
              ICS20_TRANSFER_BANK_ADDRESS=$(echo "$EVM_CONTRACTS_ARG" | jq .ics20_transfer_bank_address -r)
              ICS20_BANK_ADDRESS=$(echo "$EVM_CONTRACTS_ARG" | jq .ics20_bank_address -r)
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
                    "prover_endpoint": "'"$GALOIS_URL"'",
                    "dump_path": "'"$UNION_DUMP_PATH"'"
                  }
                }              }' | jq > "$RELAYER_CONFIG_FILE"

                deployEVMPingPong
            }

            instantiateCw20Ics20() {
              ${instantiateCw20Ics20}/bin/instantiate-cw20-ics20
            }

            instantiatePingPong() {
              ${instantiatePingPong}/bin/instantiate-ping-pong "$PING_PONG_TIMEOUT"
            }

            createClients() {
              echo ------------------------------------
              echo "+ Creating light client on evm."
              RUST_LOG=relayer=info ${self'.packages.relayer}/bin/relayer \
                --config-file-path "$RELAYER_CONFIG_FILE" \
                client create evm cometbls \
                --on ethereum-devnet \
                --counterparty union-devnet
              echo ------------------------------------
              echo "+ Creating client on union."
              RUST_LOG=relayer=info ${self'.packages.relayer}/bin/relayer \
                --config-file-path "$RELAYER_CONFIG_FILE" \
                client create union ethereum08-wasm \
                --on union-devnet \
                --counterparty ethereum-devnet \
                --evm-preset minimal
              echo ------------------------------------
            }

            waitForGaloisToBeOnline() {
              COMMAND="${self'.packages.galoisd-devnet}/bin/galoisd gen-contract $GALOIS_URL"

              if [[ -n "$GALOIS_TLS" ]]; then
                COMMAND="$COMMAND $GALOIS_TLS"
              fi

              while ! eval "$COMMAND" 1>/dev/null 2>&1
              do
                echo ".. Waiting for galois to be ready at $GALOIS_URL .."
                sleep 5
              done
            }

            setupInitialChannel() {
              MODULE_ADDRESS=$1
              PORT_ID=$2
              COUNTERPARTY_PORT_ID=$3
              CHANNEL_ID=$4

              echo ------------------------------------------------------------
              echo "+ Setting up the initial connection and channels on EVM.."
              ${self'.packages.relayer}/bin/relayer \
                --config-file-path "$RELAYER_CONFIG_FILE" \
                setup initial-channel \
                --on ethereum-devnet \
                --module-address "$MODULE_ADDRESS" \
                --channel-id "$CHANNEL_ID" \
                --port-id "$PORT_ID" \
                --counterparty-port-id "$COUNTERPARTY_PORT_ID"
              echo "+ Initial connection and channels are ready."
            }

            printIBCSetupInfo() {
              echo ---------------------------------------------------
              echo "+ Module $1(EVM) and $2(Union) is connected at:"
              echo "    - Address on EVM:   $3"
              echo "    - Address on Union: $4"
              echo "    - Connection:       $5"
              echo "    - Channel:          $6"
              echo "    - Port on EVM:      $7"
              echo "    - Port on Union:    $8"
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

            # Relayer requires the scheme to be included (http(s)) but galoisd returns an error when
            # it is run with a scheme in the URL.
            # TODO(aeryz): This should not be the case, this should probably be fixed in galois
            GALOIS_URL=$(echo "$GALOIS_URL" | sed -e "s/^http:\/\///" | sed -e "s/^https:\/\///")

            if [[ -n "$CIRCUIT_PATH" ]]; then
              runGalois &
            else
              echo "+ --circuit-path is empty, will use the galois at $GALOIS_URL"
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
            waitForGaloisToBeOnline


            echo "--------------------------------"
            echo "+ Starting the relayer.."
            echo "+ Relayer config path is: $RELAYER_CONFIG_FILE"
            printIBCSetupInfo "ICS20 Transfer" "CW20-ICS20" "$ICS20_TRANSFER_BANK_ADDRESS" "$CW20_ADDRESS"  "connection-0" "channel-0" "transfer" "wasm.$CW20_ADDRESS"
            printIBCSetupInfo "PingPong" "PingPong" "$PING_PONG_MODULE_ADDRESS" "$PING_PONG_ADDRESS" "connection-0" "channel-1" "ping-pong" "wasm.$PING_PONG_ADDRESS"

            RUST_LOG=relayer=info ${self'.packages.relayer}/bin/relayer \
              --config-file-path "$RELAYER_CONFIG_FILE" \
              relay \
              --between union-devnet:ethereum-devnet

            wait
          '';
        };
    };
}
