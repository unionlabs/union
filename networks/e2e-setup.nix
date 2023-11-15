{ ... }: {
  perSystem = { devnetConfig, pkgs, self', cw-instantiate2-salt, writeShellApplicationWithArgs, ... }:
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

    instantiateCwUCS01 =
      pkgs.writeShellApplication {
        name = "instantiate-ucs01-relay";
        runtimeInputs = [ ];
        text =
          ''
            # This account will be the governor and admin of the contract that we instantiate
            ACCOUNT_ADDRESS="$(${accountAddress})"

            INIT_MESSAGE='{
                "default_timeout":300,
                "gov_contract": "'"$ACCOUNT_ADDRESS"'",
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
                  "version":"ucs01-0",
                  "connection_id":"connection-0"
                }
              }'

            echo "$INIT_MESSAGE"

            ${instantiateContract { code-id = 1; label = "ucs01-relay"; }}
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
  {
    packages.e2e-setup =
      writeShellApplicationWithArgs {
        name = "union-devnet-demo";
        runtimeInputs = [ pkgs.jq ];
        arguments = {
          # handshake = {
          #   type = "flag";
          #   help = "Do connection/channel handshake for ping pong";
          # };
          galois_url = { 
            default = "http://0.0.0.0:16657";
            help = "Endpoint that serves galois";
          };
          galois_tls = {
            type = "flag";
            help = "Connect to galois using TLS";
          };
          circuit_path = {
            help = "Path to the circuit files to run galois locally (if not specified, galois won't be run)";
          };
          evm_beacon_rpc_url = {
            default = "http://localhost:9596";
            help = "Rpc endpoint for the evm beacon chain";
          };
          evm_ws_url = {
            default = "ws://localhost:8546";
            help = "Websocket endpoint for the evm execution chain";
          };
          union_rpc_url = {
            default = "http://localhost:26657";
            help = "Rpc endpoint for union";
          };
          union_grpc_url = {
            default = "http://localhost:9090";
            help = "gRpc endpoint for union";
          };
          union_ws_url = {
            default = "ws://localhost:26657/websocket";
            help = "Websocket endpoint for union";
          };
          voyager_config_file = {
            help = "Path to voyager config file.";
          };
          # no_run_voyager = {
          #   type = "flag";
          #   help = "Don't run voyager for packet relaying, only print the command";
          # };
          ping_pong_timeout = {
            default = "1000";
            help = "Number of blocks required for a pong message to timeout";
          };
        };
        text = ''
          # Kill all subprocesses the root process dies
          trap "kill 0" EXIT

          if [[ -z "$argc_voyager_config_file" ]]; then
            argc_voyager_config_file="$(mktemp -d)/voyager-config.json"
            echo "+ Created the voyager configuration file: $argc_voyager_config_file"
          fi

          # Check if union devnet is running
          unionAliveTest() {
            curl -sS -X GET "$argc_union_rpc_url/status" -H 'Content-Type: application/json' 1>/dev/null
          }

          # Check if eth devnet is running
          ethAliveTest() {
            curl -sS -X GET "$argc_evm_beacon_rpc_url/eth/v1/beacon/blocks/finalized" -H 'accept: application/json' 1>/dev/null
          }

          # This downloads the circuit if its not up-to date or if it doesn't exist
          downloadGaloisCircuits() {
            if [[ -n "$argc_circuit_path" ]]; then
                echo "Checking if we need to download the circuit.."
                ${self'.packages.download-circuit}/bin/download-circuit "$argc_circuit_path"
            else
              echo "Skipping downloading the circuit since --galois_url is provided."
            fi
          }

          runGalois() {
            echo Checking if circuits exists at "$argc_circuit_path"..
            if [ ! -f "$argc_circuit_path/r1cs.bin" ] || [ ! -f "$argc_circuit_path/pk.bin" ] || [ ! -f "$argc_circuit_path/vk.bin" ]; then
              echo "Some files are still missing. Please re-run the command to download the files.."
              exit 1
            fi
            echo "Starting galois.."
            

            command="${self'.packages.galoisd}/bin/galoisd serve $argc_galois_url"
            if [[ -n "$argc_galois_tls" ]]; then
              command="$command $argc_galois_tls"
            fi

            eval "$command"
          }

          evm_contracts_outfile=$(mktemp)

          deployEVMPingPong() {
            export IBC_HANDLER_ADDRESS="$ibc_handler_address"
            export NUM_OF_BLOCK_BEFORE_PONG_TIMEOUT="$argc_ping_pong_timeout"
            export REVISION_NUMBER=1

            echo ------------------------------------
            echo + Deploying Ping Pong App..
            ${self'.packages.evm-devnet-ping-pong-deploy}/bin/evm-devnet-ping-pong-deploy | tee "$evm_contracts_outfile"
            echo ------------------------------------
            evm_contracts_arg=$(tail -1 < "$evm_contracts_outfile")
            rm "$evm_contracts_outfile"

            # argc_ping_pong_module_address=$(echo "$evm_contracts_arg" | jq .ping_pong_address -r)
          }

          deployEVMContracts() {
            echo ------------------------------------
            echo + Deploying IBC Contracts..
            while ! ${self'.packages.evm-devnet-deploy}/bin/evm-devnet-deploy | tee "$evm_contracts_outfile"
            do
              echo "Eth doesn't seem to be ready yet. Will try in 3 seconds."
              sleep 3
            done
            echo ------------------------------------

            evm_contracts_arg=$(tail -1 < "$evm_contracts_outfile")
            rm "$evm_contracts_outfile"


            cometbls_client_address=$(echo "$evm_contracts_arg" | jq .cometbls_client_address -r)
            ibc_handler_address=$(echo "$evm_contracts_arg" | jq .ibc_handler_address -r)
            # ucs01relay=$(echo "$evm_contracts_arg" | jq .ucs01_relay_address -r)
            wasm_code_id=$(cat ${self'.packages.devnet-genesis}/code-ids/ethereum_light_client_minimal)
            evm_wallet=$(cat ${self'.packages.devnet-evm-config}/dev-key0.prv)

            # TODO(aeryz): fetch fee denom
            echo '{
              "chain": {
                "ethereum-devnet": {
                  "chain_type": "evm",
                  "preset_base": "minimal",
                  "ibc_handler_address": "'"$ibc_handler_address"'",
                  "signers": [{
                    "raw": "0x'"$evm_wallet"'"
                  }],
                  "eth_rpc_api": "'"$argc_evm_ws_url"'",
                  "eth_beacon_rpc_api": "'"$argc_evm_beacon_rpc_url"'"
                },
                "union-devnet": {
                  "chain_type": "union",
                  "signers": [{
                    "raw": "0xaa820fa947beb242032a41b6dc9a8b9c37d8f5fbcda0966b1ec80335b10a7d6f"
                  }],
                  "fee_denom": "stake",
                  "ws_url": "'"$argc_union_ws_url"'",
                  "prover_endpoint": "'"$argc_galois_url"'",
                  "grpc_url": "'"$argc_union_grpc_url"'"
                }
              },
              "voyager": {
                "hasura": null,
                "num_workers": 10,
                "queue": {
                  "type": "pg-queue",
                  "database_url": "postgres://postgres:postgrespassword@localhost:5432/default"
                }
              }
            }' | jq > "$argc_voyager_config_file"

              deployEVMPingPong
          }

          instantiateCwUCS01() {
            ${instantiateCwUCS01}/bin/instantiate-ucs01-relay
          }

          instantiatePingPong() {
            ${instantiatePingPong}/bin/instantiate-ping-pong "$argc_ping_pong_timeout"
          }

          createClients() {
            curl localhost:65534/msg -H 'Content-Type: application/json' -d \
              '
              {
                "Sequence": [
                  {
                    "Aggregate": {
                      "queue": [
                        {
                          "Fetch": {
                            "EthereumMinimal": {
                              "chain_id": "union-devnet-1",
                              "data": {
                                "SelfClientState": {
                                  "at": "latest"
                                }
                              }
                            }
                          }
                        },
                        {
                          "Fetch": {
                            "EthereumMinimal": {
                              "chain_id": "union-devnet-1",
                              "data": {
                                "SelfConsensusState": {
                                  "at": "latest"
                                }
                              }
                            }
                          }
                        }
                      ],
                      "data": [],
                      "receiver": {
                        "CometblsMinimal": {
                          "chain_id": "32382",
                          "data": {
                            "CreateClient": {
                              "config": {
                                "client_type": "cometbls",
                                "cometbls_client_address": "'"$cometbls_client_address"'"
                              }
                            }
                          }
                        }
                      }
                    }
                  },
                  {
                    "Aggregate": {
                      "queue": [
                        {
                          "Fetch": {
                            "CometblsMinimal": {
                              "chain_id": "32382",
                              "data": {
                                "SelfClientState": {
                                  "at": "latest"
                                }
                              }
                            }
                          }
                        },
                        {
                          "Fetch": {
                            "CometblsMinimal": {
                              "chain_id": "32382",
                              "data": {
                                "SelfConsensusState": {
                                  "at": "latest"
                                }
                              }
                            }
                          }
                        }
                      ],
                      "data": [],
                      "receiver": {
                        "EthereumMinimal": {
                          "chain_id": "union-devnet-1",
                          "data": {
                            "CreateClient": {
                              "config": {
                                "code_id": "0x'"$wasm_code_id"'"
                              }
                            }
                          }
                        }
                      }
                    }
                  }
                ]
              }
              '

            while ! ${self'.packages.voyager}/bin/voyager -c "$argc_voyager_config_file" query --on union-devnet ibc-path client-state-path 08-wasm-0
            do
              echo ".. Waiting for the client to be created on union.."
              sleep 5
            done
            echo "The client on union is created."

            while ! ${self'.packages.voyager}/bin/voyager -c "$argc_voyager_config_file" query --on ethereum-devnet ibc-path client-state-path cometbls-0
            do
              echo ".. Waiting for the client to be created on ethereum.."
              sleep 5
            done
            echo "The client on ethereum is created."
          }

          waitVoyagerToBeOnline() {
            # TODO(aeryz): Make this configurable as well
            while ! curl localhost:65534/health
            do
              echo ".. Waiting for voyager to be online. Config path: $argc_voyager_config_file"
              sleep 2
            done
          }

          downloadGaloisCircuits
          ethAliveTest
          unionAliveTest

          instantiateCwUCS01
          sleep 6
          instantiatePingPong
          deployEVMContracts
          waitVoyagerToBeOnline
          createClients

          # Voyager requires the scheme to be included (http(s)) but galoisd returns an error when
          # it is run with a scheme in the URL.
          # TODO(aeryz): This should not be the case, this should probably be fixed in galois
          argc_galois_url=$(echo "$argc_galois_url" | sed -e "s/^http:\/\///" | sed -e "s/^https:\/\///")

          if [[ -n "$argc_circuit_path" ]]; then
            runGalois &
          else
            echo "+ --circuit-path is empty, will use the galois at $argc_galois_url"
          fi

         '';
      };
  };
}
