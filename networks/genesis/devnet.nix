#cspell:ignore abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz
{ ... }: {
  perSystem = { devnetConfig, system, pkgs, self', inputs', cw-instantiate2-salt, dbg, ... }:
    let
      nodeBin = pkgs.lib.getExe self'.packages.uniond;
      chainId = "union-devnet-1";
      chainName = "union";

      devKeyMnemonics = {
        alice = "wine parrot nominee girl exchange element pudding grow area twenty next junior come render shadow evidence sentence start rough debate feed all limb real";
        bob = "gun more barrel helmet velvet people alter depth bargain use isolate pear before frown already limb sweet response legal invest stand barrel stone conduct";
        charlie = "young soup enroll tornado mercy athlete tray resist limit spare address license cargo quantum panda useful clog autumn shoot observe input next across movie";
        dave = "allow where void replace vocal cabbage can expose rival danger stomach noodle faculty cart surround cash rice kite audit slight ten bicycle dance middle";
        erin = "hard educate knock ketchup salon obey debate one other impose smoke spoon pull describe cactus talk other merit joy great critic canvas scene lounge";
        frank = "over floor explain will stereo camera subway park pilot trick good exchange foot violin shop kite educate bracket shoulder fancy denial ill era battle";
      };

      mkNodeMnemonic = idx:
        assert (builtins.isInt idx);
        pkgs.runCommand
          "${chainName}-mnemonic_${toString idx}"
          { buildInputs = [ self'.packages.keygen ]; }
          ''
            keygen mnemonic $(echo ${toString idx} | sha256sum - | cut -d' ' -f1) > $out

            echo "validator ${toString idx} mnemonic: $(cat $out)"
          '';

      mkNodeKey = idx:
        assert (builtins.isInt idx);
        pkgs.runCommand
          "${chainName}-node-key_${toString idx}"
          { buildInputs = [ self'.packages.keygen ]; }
          ''
            NODE_KEY=$(keygen key --key-type ed25519 "$(cat ${mkNodeMnemonic idx})" | tr -d '\n')

            echo "validator ${toString idx} node_key: $NODE_KEY"

            echo "{\"priv_key\":{\"type\":\"tendermint/PrivKeyEd25519\",\"value\":\"$NODE_KEY\"}}" > $out
          '';

      mkNodeId = idx:
        assert (builtins.isInt idx);
        pkgs.runCommand
          "${chainId}-node-id_${toString idx}"
          { }
          ''
            export HOME=$(pwd)

            cat ${mkNodeMnemonic idx} | ${nodeBin} \
              init \
              testnet \
              --chain-id ${chainId} \
              --home . \
              --recover 2> /dev/null

            cp ${mkNodeKey idx} ./config/node_key.json
            ${nodeBin} tendermint show-node-id --home . | tr -d '\n' > $out
          '';

      mkPrivValidatorKey = idx:
        assert (builtins.isInt idx);
        pkgs.runCommand
          "${chainName}-priv-validator-key_${toString idx}"
          { }
          ''
            export HOME=$(pwd)

            cat ${mkNodeMnemonic idx} | ${nodeBin} \
              init \
              testnet \
              --chain-id ${chainId} \
              --home . \
              --recover 2> /dev/null

            mv ./config/priv_validator_key.json $out
            echo "created valkey-${toString idx}: $(cat $out)"
          '';

      mkValGentx = idx:
        dbg (pkgs.runCommand
          "${chainName}-valgentx_${toString idx}"
          {
            buildInputs = [ pkgs.jq ];
            src = addAllKeysToKeyringAndGenesis initHome;
          }
          ''
            export HOME=$(pwd)

            PUBKEY="{\"@type\":\"/cosmos.crypto.bn254.PubKey\",\"key\":$(cat ${mkPrivValidatorKey idx} | jq ."pub_key"."value")}"

            echo "validator pubkey: $PUBKEY"

              # --offline \
              # --account-number ${toString (valoperAccountNumber idx)} \
              # --sequence 0 \

            ${nodeBin} \
              genesis \
              gentx \
              valoper-${toString idx} \
              1000000000000000000000stake \
              --chain-id ${chainId} \
              --home $src \
              --keyring-backend test \
              --ip "0.0.0.0" \
              --pubkey "$PUBKEY" \
              --moniker validator-${toString idx} \
              --output-document $out
          '');

      valoperAccountNumber = idx: (builtins.length (builtins.attrValues devKeyMnemonics)) + idx - 1;

      initHome = pkgs.runCommand
        "${chainName}-genesis-home"
        {
          buildInputs = [ pkgs.jq pkgs.moreutils ];
        }
        ''
          export HOME=$(pwd)
          mkdir -p $out

          ${nodeBin} init testnet --chain-id ${chainId} --home $out
        '';

      addDevKeyToKeyringAndGenesis = name: mnemonic: home:
        pkgs.runCommand
          "${chainName}-add-dev-key-${name}"
          {
            buildInputs = [ pkgs.jq pkgs.moreutils ];
          }
          ''
            export HOME=$(pwd)
            mkdir -p $out
            cp --no-preserve=mode -r ${home}/* $out

            # Add the dev account
            echo ${mnemonic} | ${nodeBin} keys add \
              --recover ${name} \
              --keyring-backend test \
              --home $out

            ${nodeBin} \
              genesis \
              add-genesis-account \
              ${name} \
              10000000000000000000000000stake \
              --keyring-backend test \
              --home $out
          '';

      addValoperKeyToKeyringAndGenesis = idx: home:
        assert (builtins.isInt idx);
        pkgs.runCommand
          "${chainName}-valkey_${toString idx}"
          { }
          ''
            export HOME=$(pwd)
            mkdir -p $out
            cp --no-preserve=mode -r ${home}/* $out

            cat ${mkNodeMnemonic idx} | ${nodeBin} keys add \
              --recover valoper-${toString idx} \
              --keyring-backend test \
              --home $out

            ${nodeBin} \
              genesis \
              add-genesis-account \
              valoper-${toString idx} \
              10000000000000000000000000stake \
              --keyring-backend test \
              --home $out
          '';

      addAllKeysToKeyringAndGenesis = home: pkgs.lib.foldl
        (home: f: (dbg f) (dbg home))
        initHome
        (dbg (
          pkgs.lib.flatten [
            (pkgs.lib.mapAttrsToList addDevKeyToKeyringAndGenesis devKeyMnemonics)

            (dbg (builtins.genList addValoperKeyToKeyringAndGenesis devnetConfig.validatorCount))
          ]
        ));

      applyGenesisOverwrites = genesisOverwrites: home:
        let
          overwrites = builtins.toFile "overwrite.json" (builtins.toJSON genesisOverwrites);
        in
        pkgs.runCommand "${chainName}-apply-genesis-overwrites"
          {
            buildInputs = [ pkgs.jq ];
          }
          ''
            mkdir -p $out
            cp --no-preserve=mode -r ${home}/* $out
            jq -s '.[0] * .[1]' ${home}/config/genesis.json ${overwrites} > merge.json
            mv merge.json $out/config/genesis.json
          '';

      # calculateCw20Ics20ContractAddress = home: pkgs.runCommand "calculate-ucs01-relay-contract-address"
      #   {
      #     buildInputs = [ pkgs.jq ];
      #   }
      #   ''
      #     export HOME=$(pwd)
      #     mkdir -p $out
      #     cp --no-preserve=mode -r ${home}/* $out

      #     ALICE_ADDRESS=$(${nodeBin} keys list \
      #       --keyring-backend test \
      #       --home $out \
      #       --output json \
      #       | jq '.[] | select(.name == "alice").address' --raw-output)

      #     CODE_HASH=$(sha256sum ${self'.packages.ucs01-relay}/lib/ucs01_relay.wasm | cut -f1 -d" ")

      #     ${nodeBin} query wasm build-address $CODE_HASH $ALICE_ADDRESS ${cw-instantiate2-salt} --home $out > $out/CW20_ICS20_CONTRACT_ADDRESS
      #   '';

      # calculatePingPongAddress = home: pkgs.runCommand "calculate-ping-pong-contract-address"
      #   {
      #     buildInputs = [ pkgs.jq ];
      #   }
      #   ''
      #     export HOME=$(pwd)
      #     mkdir -p $out
      #     cp --no-preserve=mode -r ${home}/* $out

      #     ALICE_ADDRESS=$(${nodeBin} keys list \
      #       --keyring-backend test \
      #       --home $out \
      #       --output json \
      #       | jq '.[] | select(.name == "alice").address' --raw-output)

      #     CODE_HASH=$(sha256sum ${self'.packages.ucs00-pingpong}/lib/ucs00_pingpong.wasm | cut -f1 -d" ")

      #     ${nodeBin} query wasm build-address $CODE_HASH $ALICE_ADDRESS ${cw-instantiate2-salt} --home $out > $out/PING_PONG_CONTRACT_ADDRESS
      #   '';

      # addIbcConnectionToGenesis = home: pkgs.runCommand "add-ibc-connection-to-genesis"
      #   {
      #     buildInputs = [ pkgs.jq pkgs.moreutils ];
      #   }
      #   ''
      #     export HOME=$(pwd)
      #     mkdir -p $out
      #     cp --no-preserve=mode -r ${home}/* $out

      #     jq \
      #      '.app_state.ibc.connection_genesis.connections += [{
      #         "id": "connection-0",
      #         "client_id": "08-wasm-0",
      #         "versions": [{
      #           "identifier": "1",
      #           "features": [
      #             "ORDER_ORDERED", "ORDER_UNORDERED"
      #           ]
      #          }],
      #         "state": 3,
      #         "counterparty": {
      #           "client_id": "cometbls-new-0",
      #           "connection_id": "connection-0",
      #           "prefix": {
      #             "key_prefix": "aWJj"
      #           }
      #         },
      #         "delay_period": 0
      #       }]' \
      #       $out/config/genesis.json | sponge $out/config/genesis.json

      #     jq \
      #       '.app_state.ibc.connection_genesis.client_connection_paths += [{
      #           "client_id": "08-wasm-0",
      #           "paths": ["connection-0"]
      #       }]' \
      #       $out/config/genesis.json | sponge $out/config/genesis.json

      #     # Connection id sequence is advanced to prevent overlapping.
      #     jq \
      #       '.app_state.ibc.connection_genesis.next_connection_sequence = "1"' \
      #       $out/config/genesis.json | sponge $out/config/genesis.json
      #   '';

      add08WasmToAllowedClients = home: pkgs.runCommand "${chainName}-add-ibc-connection-to-genesis"
        {
          buildInputs = [ pkgs.jq pkgs.moreutils ];
        }
        ''
          export HOME=$(pwd)
          mkdir -p $out
          cp --no-preserve=mode -r ${home}/* $out

          jq \
           '.app_state.ibc.client_genesis.params.allowed_clients += ["08-wasm"]' \
            $out/config/genesis.json | sponge $out/config/genesis.json
        '';


      # addIbcChannelToGenesis = home: pkgs.runCommand "add-ibc-channel-to-genesis"
      #   {
      #     buildInputs = [ pkgs.jq pkgs.moreutils ];
      #   }
      #   ''
      #     export HOME=$(pwd)
      #     mkdir -p $out
      #     cp --no-preserve=mode -r ${home}/* $out

      #     ALICE_ADDRESS=$(${nodeBin} keys list \
      #       --keyring-backend test \
      #       --home $out \
      #       --output json \
      #       | jq '.[] | select(.name == "alice").address' --raw-output)

      #     CW20_ADDRESS=$(cat ${calculateCw20Ics20ContractAddress home}/CW20_ICS20_CONTRACT_ADDRESS)
      #     CW20_PORT=wasm.$CW20_ADDRESS

      #     PING_PONG_ADDRESS=$(cat ${calculatePingPongAddress home}/PING_PONG_CONTRACT_ADDRESS)
      #     PING_PONG_PORT=wasm.$PING_PONG_ADDRESS

      #     # TODO(aeryz): get the port id and channel info as parameters
      #     jq \
      #      --arg cw20_port $CW20_PORT \
      #      '.app_state.ibc.channel_genesis.channels += [{
      #         "state": 3,
      #         "ordering": 1,
      #         "counterparty": {
      #           "port_id": "transfer",
      #           "channel_id": "channel-0"
      #         },
      #         "connection_hops": ["connection-0"],
      #         "version": "ics20-1",
      #         "port_id": $cw20_port,
      #         "channel_id": "channel-0"
      #       }]' \
      #       $out/config/genesis.json | sponge $out/config/genesis.json

      #     jq \
      #      --arg cw20_port $CW20_PORT \
      #      --arg ping_pong_port $PING_PONG_PORT \
      #      '.app_state.ibc.channel_genesis.send_sequences += [{
      #         "port_id": $cw20_port,
      #         "channel_id": "channel-0",
      #         "sequence": "1"
      #       }, {
      #         "port_id": $ping_pong_port,
      #         "channel_id": "channel-1",
      #         "sequence": "1"
      #       }]' \
      #       $out/config/genesis.json | sponge $out/config/genesis.json

      #     jq \
      #      --arg cw20_port $CW20_PORT \
      #      --arg ping_pong_port $PING_PONG_PORT \
      #      '.app_state.ibc.channel_genesis.recv_sequences += [{
      #         "port_id": $cw20_port,
      #         "channel_id": "channel-0",
      #         "sequence": "1"
      #       }, {
      #         "port_id": $ping_pong_port,
      #         "channel_id": "channel-1",
      #         "sequence": "1"
      #       }]' \
      #       $out/config/genesis.json | sponge $out/config/genesis.json

      #     jq \
      #      --arg cw20_port $CW20_PORT \
      #      --arg ping_pong_port $PING_PONG_PORT \
      #      '.app_state.ibc.channel_genesis.ack_sequences += [{
      #         "port_id": $cw20_port,
      #         "channel_id": "channel-0",
      #         "sequence": "1"
      #       }, {
      #         "port_id": $ping_pong_port,
      #         "channel_id": "channel-1",
      #         "sequence": "1"
      #       }]' \
      #       $out/config/genesis.json | sponge $out/config/genesis.json

      #     jq \
      #      --arg ping_pong_port $PING_PONG_PORT \
      #      '.app_state.ibc.channel_genesis.channels += [{
      #         "state": 3,
      #         "ordering": 1,
      #         "counterparty": {
      #           "port_id": "ping-pong",
      #           "channel_id": "channel-1"
      #         },
      #         "connection_hops": ["connection-0"],
      #         "version": "ics20-1",
      #         "port_id": $ping_pong_port,
      #         "channel_id": "channel-1"
      #       }]' \
      #       $out/config/genesis.json | sponge $out/config/genesis.json

      #     jq \
      #       '.app_state.ibc.channel_genesis.next_channel_sequence = "2"' \
      #       $out/config/genesis.json | sponge $out/config/genesis.json

      #     jq \
      #       '.app_state.capability.index = "3"' \
      #       $out/config/genesis.json | sponge $out/config/genesis.json

      #     jq \
      #      --arg capability1 capabilities/ports/$CW20_PORT/channels/channel-0 \
      #      --arg capability2 capabilities/ports/$PING_PONG_PORT/channels/channel-1 \
      #       '.app_state.capability.owners += [{
      #         "index": "1",
      #         "index_owners": {
      #           "owners": [
      #             {
      #               "module": "ibc",
      #               "name": $capability1
      #             },
      #             {
      #               "module": "wasm",
      #               "name": $capability1
      #             }
      #           ]
      #         }
      #       },{
      #         "index": "2",
      #         "index_owners": {
      #           "owners": [
      #             {
      #               "module": "ibc",
      #               "name": $capability2
      #             },
      #             {
      #               "module": "wasm",
      #               "name": $capability2
      #             }
      #           ]
      #         }
      #       } ]' \
      #       $out/config/genesis.json | sponge $out/config/genesis.json

      #   '';

      addLightClientCodeToGenesis = contract: home:
        pkgs.runCommand
          "${chainName}-add-light-client-contract-code-to-genesis"
          {
            buildInputs = [ pkgs.jq pkgs.moreutils ];
          }
          ''
            export HOME=$(pwd)
            mkdir -p $out
            cp --no-preserve=mode -r ${home}/* $out

            # Generate the wasm client genesis state
            for wasm in $(find ${contract} -name "*.wasm" -type f); do
              echo "adding $wasm to genesis"

              base64 -w0 $wasm > encoded
              CHECKSUM=$(sha256sum $wasm | cut -f1 -d " ")
              # CODE_ID=$(echo -ne "codeId/$CHECKSUM" | base64 -w0)

              # echo "code id is '$CODE_ID'"

              mkdir -p $out/code-ids
              echo "$CHECKSUM" > "$out/code-ids/$(basename $wasm .wasm)"

               jq \
                --rawfile encoded_file encoded \
                '.app_state."08-wasm".contracts += [{
                  "code_bytes": $encoded_file
                }]' \
                $out/config/genesis.json | sponge $out/config/genesis.json
            done
          '';

      addIbcContractCodesToGenesis = contracts: home:
        let
          addContract = { contract, idx }: home:
            pkgs.runCommand
              "${chainName}-add-ibc-contract-code-to-genesis"
              {
                buildInputs = [ pkgs.jq pkgs.moreutils pkgs.xxd ];
              }
              ''
                export HOME=$(pwd)
                mkdir -p $out
                cp --no-preserve=mode -r ${home}/* $out

                ALICE_ADDRESS=$(${nodeBin} keys list \
                  --keyring-backend test \
                  --home $out \
                  --output json \
                  | jq '.[] | select(.name == "alice").address' --raw-output)

                for wasm in $(find ${contract} -name "*.wasm" -type f); do
                  echo "adding $wasm to genesis"

                  echo "code id is '${toString idx}'"

                  mkdir -p $out/code-ids
                  echo "${toString idx}" > "$out/code-ids/$(basename $wasm .wasm)"

                  jq \
                    --arg code_hash $(sha256sum $wasm | cut -f1 -d" " | xxd -r -p | base64) \
                    --arg last_code_id_key $(echo -ne "\x04lastCodeId" | base64) \
                    --arg creator $ALICE_ADDRESS \
                    --rawfile encoded <(base64 -w0 $wasm) \
                    '.app_state.wasm.codes += [{
                      "code_id": "${toString idx}",
                      "code_info": {
                        "code_hash": $code_hash,
                        "creator": $creator,
                        "instantiate_config": { "permission": "Everybody" }
                      },
                      "code_bytes": $encoded,
                      "pinned": false
                    }]' \
                    $out/config/genesis.json | sponge $out/config/genesis.json
                done
              '';

          home' = (pkgs.lib.foldl
            (h: contract: addContract contract h)
            home
            (pkgs.lib.imap1 (idx: c: { contract = c; inherit idx; }) contracts));
        in
        pkgs.runCommand
          "${chainName}-add-ibc-contract-codes-to-genesis"
          {
            buildInputs = [ pkgs.jq pkgs.moreutils ];
          }
          ''
            export HOME=$(pwd)
            mkdir -p $out
            cp --no-preserve=mode -r ${home'}/* $out

              jq \
                --arg last_code_id_key $(echo -ne "\x04lastCodeId" | base64) \
                '.app_state.wasm.sequences += [{
                  "id_key": $last_code_id_key,
                  "value": "${toString ((builtins.length contracts) + 1)}",
                }]' \
                $out/config/genesis.json | sponge $out/config/genesis.json
          '';

      genesisHome = pkgs.lib.foldl
        (home: f: (dbg f) (dbg home))
        initHome
        (
          pkgs.lib.flatten [
            addAllKeysToKeyringAndGenesis

            (applyGenesisOverwrites devnetConfig.genesisOverwrites)

            # add light clients
            (builtins.map addLightClientCodeToGenesis [
              self'.packages.ethereum-light-client-minimal
              self'.packages.ethereum-light-client-mainnet
            ])

            # add ibc contracts
            add08WasmToAllowedClients

            (addIbcContractCodesToGenesis [
              self'.packages.ucs01-relay
              self'.packages.ucs00-pingpong
            ])

            # add ibc connection
            # addIbcConnectionToGenesis
            # addIbcChannelToGenesis
          ]
        );
    in
    {
      packages."devnet-${chainName}-genesis" = pkgs.runCommand "${chainName}-genesis" { } ''
        mkdir $out
        cd $out

        export HOME=$(pwd)

        # Copy the read-only genesis we used to build the genesis file as the collect-gentxs command will overwrite it
        cp --no-preserve=mode -r ${genesisHome}/* .

        mkdir ./config/gentx
        ${builtins.concatStringsSep "\n" (builtins.genList (idx: ''
          cp ${mkValGentx idx} ./config/gentx/valgentx-${toString idx}.json
        '') devnetConfig.validatorCount)}

        echo "collecting"
        ${nodeBin} genesis collect-gentxs --home . 2> /dev/null

        echo "validating"
        ${nodeBin} genesis validate --home .
      '';

      packages."devnet-${chainName}-priv-validator-keys" = pkgs.linkFarm
        "devnet-${chainName}-priv-validator-keys"
        (builtins.genList
          (idx: {
            name = "priv_validator_key_${toString idx}.json";
            path = mkPrivValidatorKey idx;
          })
          devnetConfig.validatorCount);

      packages."devnet-${chainName}-validator-gentxs" = pkgs.linkFarm
        "devnet-${chainName}-validator-gentxs"
        (builtins.genList
          (idx: {
            name = "gentx-${toString idx}.json";
            path = mkValGentx idx;
          })
          devnetConfig.validatorCount);

      packages."devnet-${chainName}-validator-node-keys" = pkgs.linkFarm
        "devnet-${chainName}-validator-node-keys"
        (builtins.genList
          (idx: {
            name = "node-key-${toString idx}.json";
            path = mkNodeKey idx;
          })
          devnetConfig.validatorCount);

      packages."devnet-${chainName}-validator-node-ids" = pkgs.linkFarm
        "devnet-${chainName}-validator-node-ids"
        (builtins.genList
          (idx: {
            name = "node-id-${toString idx}";
            path = mkNodeId idx;
          })
          devnetConfig.validatorCount);

      # FIXME: This shouldn't be defined in this file
      packages.devnet-eth-config = pkgs.linkFarm "devnet-eth-config" [
        { name = "genesis.json"; path = "${./devnet-eth/genesis.json}"; }
        { name = "dev-key0.prv"; path = "${./devnet-eth/dev-key0.prv}"; }
        { name = "dev-key1.prv"; path = "${./devnet-eth/dev-key1.prv}"; }
        { name = "dev-jwt.prv"; path = "${./devnet-eth/dev-jwt.prv}"; }
      ];

      _module.args.mkNodeId = mkNodeId;

      checks = { };
    };
}

