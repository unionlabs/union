{ ... }: {
  perSystem = { devnetConfig, pkgs, self', inputs', ... }:
    let
    uniond = pkgs.lib.getExe self'.packages.uniond;

    instantiate-cw20-ics20 =
      pkgs.writeShellApplication {
        name = "instantiate-cw20-ics20";
        runtimeInputs = [];
        text =
      ''
        while ! ${uniond} tx wasm instantiate2 1 \
          '{
            "default_timeout":300,
            "gov_contract":"union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2",
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
          61616161 \
          --label cw20-ics20-test \
          --gas=auto \
          --gas-adjustment=1.3 -y  \
          --admin union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2 \
          --keyring-backend test \
          --from testkey \
          --chain-id union-devnet-1 \
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
            trap "kill 0" EXIT
            TMP_HOME=$(mktemp -d)

            GALOIS_URL=""
            CIRCUIT_PATH=""

            while [[ $# -gt 0 ]]; do
              case $1 in
                -g|--galois-url)
                  GALOIS_URL="$2"
                  shift # past argument
                  shift # past value
                  ;;
                -c|--circuit-path)
                  CIRCUIT_PATH="$2"
                  shift
                  shift
                  ;;
              esac
            done

            # Check if both devnets are deployed
            unionAliveTest() {
              curl -sS -X GET 'http://localhost:26657/status' -H 'Content-Type: application/json' 1>/dev/null
            }

            ethAliveTest() {
              curl -sS -X GET 'http://localhost:9596/eth/v1/beacon/blocks/finalized' -H 'accept: application/json' 1>/dev/null
            }

            downloadGalois() {
              if [[ -z "$GALOIS_URL" ]]; then
                if [ ! -f "$CIRCUIT_PATH/r1cs.bin" ] || [ ! -f "$CIRCUIT_PATH/pk.bin" ] || [ ! -f "$CIRCUIT_PATH/vk.bin" ]; then
                  echo "Some files are missing. Downloading the circuit.."
                  ${self'.packages.download-circuit}/bin/download-circuit devnet "$CIRCUIT_PATH"
                fi                  
              else
                echo "Skipping downloading the circuit since galois url is provided."
              fi
            }

            runGalois() {
              if [[ -z "$GALOIS_URL" ]]; then
                if [[ -z "$CIRCUIT_PATH" ]]; then
                  echo When galois will be run locally, \"--circuit-path\" has to be provided. If you want to use an \
                    already running galois, provide the endpoint with \"--galois-url\".
                  exit 1
                else
                  GALOIS_URL="0.0.0.0:16657"
                  echo Checking if circuit exists in "$CIRCUIT_PATH"
                  if [ ! -f "$CIRCUIT_PATH/r1cs.bin" ] || [ ! -f "$CIRCUIT_PATH/pk.bin" ] || [ ! -f "$CIRCUIT_PATH/vk.bin" ]; then
                    echo "Some files are still missing. Please re-run the command to download the files.."
                    exit 1
                  fi                  
                  echo "Starting galois."
                  ${self'.packages.galoisd-devnet}/bin/galoisd serve "$GALOIS_URL"
                fi
              else
                echo Will use the provided galois since "--galois-url: $GALOIS_URL" is provided.
              fi
            }

            EVM_CONTRACTS_OUTFILE=$(mktemp)
            deployEVMContracts() {
              while ! ${self'.packages.evm-devnet-deploy}/bin/evm-devnet-deploy | tee "$EVM_CONTRACTS_OUTFILE" 
              do
                sleep 1
              done

              EVM_CONTRACTS_ARG=$(tail -1 < "$EVM_CONTRACTS_OUTFILE")
              rm "$EVM_CONTRACTS_OUTFILE"
              COUNTERPARTY_PORT_ID=$(${uniond} query wasm list-contract-by-code 1  --output json | jq '.contracts[0]' -r)
  
              if [[ -z "$COUNTERPARTY_PORT_ID" ]]; then
                echo "Couldn't find the uploaded contract. Something is wrong. Please start over."
                exit 0
              fi;

              COUNTERPARTY_PORT_ID="wasm.$COUNTERPARTY_PORT_ID"

              COMETBLS_ADDRESS=$(echo "$EVM_CONTRACTS_ARG" | jq .cometbls_client_address -r)
              IBC_HANDLER_ADDRESS=$(echo "$EVM_CONTRACTS_ARG" | jq .ibc_handler_address -r)
              ICS20_TRANSFER_BANK_ADDRESS=$(echo "$EVM_CONTRACTS_ARG" | jq .ics20_transfer_bank_address -r)
              ICS20_BANK_ADDRESS=$(echo "$EVM_CONTRACTS_ARG" | jq .ics20_bank_address -r)
              WASM_CODE_ID=$(cat ${self'.packages.devnet-genesis}/code-ids/ethereum_light_client_minimal)

              echo "{ \
                \"chain\": { \
                  \"ethereum-devnet\": { \
                    \"chain_type\": \"evm\", \
                    \"preset_base\": \"minimal\", \
                    \"cometbls_client_address\": \"$COMETBLS_ADDRESS\", \
                    \"ibc_handler_address\": \"$IBC_HANDLER_ADDRESS\", \
                    \"ics20_transfer_bank_address\": \"$ICS20_TRANSFER_BANK_ADDRESS\", \
                    \"ics20_bank_address\": \"$ICS20_BANK_ADDRESS\", \
                    \"signer\": { \
                      \"raw\": \"0x4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77\" \
                    }, \
                    \"eth_rpc_api\": \"ws://localhost:8546\", \
                    \"eth_beacon_rpc_api\": \"http://localhost:9596\", \
                    \"wasm_code_id\": \"0x$WASM_CODE_ID\" \
                  }, \
                  \"union-devnet\": { \
                    \"chain_type\": \"union\", \
                    \"signer\": { \
                      \"raw\": \"0xaa820fa947beb242032a41b6dc9a8b9c37d8f5fbcda0966b1ec80335b10a7d6f\" \
                    }, \
                    \"ws_url\": \"ws://127.0.0.1:26657/websocket\", \
                    \"wasm_code_id\": \"0x$WASM_CODE_ID\", \
                    \"prover_endpoint\": \"http://0.0.0.0:16657\" \
                  } \
                } \
              }" > "$TMP_HOME/relayer-config.json"

              echo "Setting up the initial connection and channel on EVM.."
              ${self'.packages.relayer}/bin/relayer \
                --config-file-path "$TMP_HOME/relayer-config.json" \
                setup initial-channel \
                --wallet 0x4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77 \
                --eth-rpc-api http://localhost:8545 \
                --ibc-handler-address "$IBC_HANDLER_ADDRESS" \
                --ics20-transfer-address "$ICS20_TRANSFER_BANK_ADDRESS" \
                --counterparty-port-id "$COUNTERPARTY_PORT_ID"
            }

            instantiateCw20Ics20() {
              ${instantiate-cw20-ics20}/bin/instantiate-cw20-ics20
            }

            createClients() {
              echo "Creating light client on evm."
              RUST_LOG=relayer=info ${self'.packages.relayer}/bin/relayer \
                --config-file-path "$TMP_HOME/relayer-config.json" \
                client create evm cometbls \
                --on ethereum-devnet \
                --counterparty union-devnet

              echo "Creating client on union."
              RUST_LOG=relayer=info ${self'.packages.relayer}/bin/relayer \
                --config-file-path "$TMP_HOME/relayer-config.json" \
                client create union ethereum08-wasm \
                --on union-devnet \
                --counterparty ethereum-devnet \
                --evm-preset minimal
            }

            startRelaying() {
              while ! ${self'.packages.galoisd-devnet}/bin/galoisd query-stats 0.0.0.0:16657
              do 
                echo "Waiting for galois to be ready at 0.0.0.0:16657.."
                sleep 2 
              done

              echo "Starting the relayer.."
              echo "+ Relayer config path is: $TMP_HOME/relayer-config.json"
              echo "+ cw20-ics20 port id is: $COUNTERPARTY_PORT_ID"
              echo "+ Channel and connection ids on both chains are: channel-0 and connection-0"

              RUST_LOG=relayer=info ${self'.packages.relayer}/bin/relayer \
                --config-file-path "$TMP_HOME/relayer-config.json" \
                relay \
                --between union-devnet:ethereum-devnet
            }

            downloadGalois
            ethAliveTest
            unionAliveTest
            instantiateCw20Ics20
            deployEVMContracts


            runGalois &
            createClients
            startRelaying

            wait
            exit 0

            # (trap 'kill 0' SIGINT; \
            #   {self'.packages.devnet}/bin/union-devnet & \
            #   {instantiate-cw20-ics20}/bin/instantiate-cw20-ics20 & \
            #   {deploy-contracts}/bin/deploy-contracts & \
            #   wait)
          '';
        };
    };
}
