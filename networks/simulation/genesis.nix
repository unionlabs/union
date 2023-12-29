#cspell:ignore abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz
{ ... }: {
  perSystem = { devnetConfig, system, pkgs, self', inputs', cw-instantiate2-salt, dbg, ... }:
    let
      alice = "wine parrot nominee girl exchange element pudding grow area twenty next junior come render shadow evidence sentence start rough debate feed all limb real";
      devMnemonics = [
        "gun more barrel helmet velvet people alter depth bargain use isolate pear before frown already limb sweet response legal invest stand barrel stone conduct"
        "young soup enroll tornado mercy athlete tray resist limit spare address license cargo quantum panda useful clog autumn shoot observe input next across movie"
        "allow where void replace vocal cabbage can expose rival danger stomach noodle faculty cart surround cash rice kite audit slight ten bicycle dance middle"
        "hard educate knock ketchup salon obey debate one other impose smoke spoon pull describe cactus talk other merit joy great critic canvas scene lounge"
        "over floor explain will stereo camera subway park pilot trick good exchange foot violin shop kite educate bracket shoulder fancy denial ill era battle"
        "mercy animal rival black process document great armor demand shiver unit lava sorry outside thank verb term you output unit thank manual spike capital"
        "embark smoke reduce belt bar mimic bench priority crop fetch portion sadness obscure around wait injury annual enable universe citizen cream blossom across dash"
        "april orbit comfort fossil clean vague exclude live enjoy bus leader sail supply bird jungle start sunny lens ensure lunch weasel merge daughter capital"
        "rebuild equip basket entire hurt index chase camera gravity pave boat vendor hero pizza october narrow train spoon cage intact jazz suffer ten spirit"
      ];
      genesisAccountName = "testkey";
      simd = pkgs.lib.getExe self'.packages.simd;
      chainId = "simd-devnet-1";
      mkNodeID = name:
        pkgs.runCommand "node-id" { } ''
          export HOME=$(pwd)

          ${simd} init testnet --chain-id ${chainId} --home .
          mkdir -p $out
          mv ./config/node_key.json $out/${name}
        '';

      initHome = pkgs.runCommand "genesis-home"
        {
          buildInputs = [ pkgs.jq pkgs.moreutils ];
        }
        ''
          export HOME=$(pwd)
          mkdir -p $out

          ${simd} init testnet --chain-id ${chainId} --home $out

          echo ${alice} | ${simd} keys add \
            --recover ${genesisAccountName} \
            --keyring-backend test \
            --home $out

          ${simd} genesis add-genesis-account ${genesisAccountName} 10000000000000000000000000stake \
            --keyring-backend test \
            --home $out

          ${builtins.concatStringsSep "\n" (pkgs.lib.lists.imap0 (i: mnemonic: ''
            # Add the dev account
            echo ${mnemonic} | ${simd} keys add \
              --recover ${genesisAccountName}-${toString i} \
              --keyring-backend test \
              --home $out
            ${simd} genesis add-genesis-account ${genesisAccountName}-${toString i} 10000000000000000000000000stake \
              --keyring-backend test \
              --home $out
          '') devMnemonics)}
        '';

      calculateCw20Ics20ContractAddress = home: pkgs.runCommand "calculate-ucs01-relay-contract-address"
        {
          buildInputs = [ pkgs.jq ];
        }
        ''
          export HOME=$(pwd)
          mkdir -p $out
          cp --no-preserve=mode -r ${home}/* $out

          ALICE_ADDRESS=$(${simd} keys list \
            --keyring-backend test \
            --home $out \
            --output json \
            | jq '.[] | select(.name == "${genesisAccountName}").address' --raw-output)

          CODE_HASH=$(sha256sum ${self'.packages.ucs01-relay}/lib/ucs01_relay.wasm | cut -f1 -d" ")

          ${simd} query wasm build-address $CODE_HASH $ALICE_ADDRESS ${cw-instantiate2-salt} > $out/CW20_ICS20_CONTRACT_ADDRESS
        '';

      calculatePingPongAddress = home: pkgs.runCommand "calculate-ping-pong-contract-address"
        {
          buildInputs = [ pkgs.jq ];
        }
        ''
          export HOME=$(pwd)
          mkdir -p $out
          cp --no-preserve=mode -r ${home}/* $out

          ALICE_ADDRESS=$(${simd} keys list \
            --keyring-backend test \
            --home $out \
            --output json \
            | jq '.[] | select(.name == "${genesisAccountName}").address' --raw-output)

          CODE_HASH=$(sha256sum ${self'.packages.ucs00-pingpong}/lib/ucs00_pingpong.wasm | cut -f1 -d" ")

          ${simd} query wasm build-address $CODE_HASH $ALICE_ADDRESS ${cw-instantiate2-salt} > $out/PING_PONG_CONTRACT_ADDRESS
        '';

      addIbcConnectionToGenesis = home: pkgs.runCommand "add-ibc-connection-to-genesis"
        {
          buildInputs = [ pkgs.jq pkgs.moreutils ];
        }
        ''
          export HOME=$(pwd)
          mkdir -p $out
          cp --no-preserve=mode -r ${home}/* $out

          jq \
           '.app_state.ibc.connection_genesis.connections += [{
              "id": "connection-0",
              "client_id": "08-wasm-0",
              "versions": [{
                "identifier": "1",
                "features": [
                  "ORDER_ORDERED", "ORDER_UNORDERED"
                ]
               }],
              "state": 3,
              "counterparty": {
                "client_id": "cometbls-new-0",
                "connection_id": "connection-0",
                "prefix": {
                  "key_prefix": "aWJj"
                }
              },
              "delay_period": 0
            }]' \
            $out/config/genesis.json | sponge $out/config/genesis.json

          jq \
            '.app_state.ibc.connection_genesis.client_connection_paths += [{
                "client_id": "08-wasm-0",
                "paths": ["connection-0"]
            }]' \
            $out/config/genesis.json | sponge $out/config/genesis.json

          # Connection id sequence is advanced to prevent overlapping.
          jq \
            '.app_state.ibc.connection_genesis.next_connection_sequence = "1"' \
            $out/config/genesis.json | sponge $out/config/genesis.json
        '';

      add08WasmToAllowedClients = home: pkgs.runCommand "add-ibc-connection-to-genesis"
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

      addIbcChannelToGenesis = home: pkgs.runCommand "add-ibc-channel-to-genesis"
        {
          buildInputs = [ pkgs.jq pkgs.moreutils ];
        }
        ''
          export HOME=$(pwd)
          mkdir -p $out
          cp --no-preserve=mode -r ${home}/* $out

          ALICE_ADDRESS=$(${simd} keys list \
            --keyring-backend test \
            --home $out \
            --output json \
            | jq '.[] | select(.name == "${genesisAccountName}").address' --raw-output)

          CW20_ADDRESS=$(cat ${calculateCw20Ics20ContractAddress home}/CW20_ICS20_CONTRACT_ADDRESS)
          CW20_PORT=wasm.$CW20_ADDRESS

          PING_PONG_ADDRESS=$(cat ${calculatePingPongAddress home}/PING_PONG_CONTRACT_ADDRESS)
          PING_PONG_PORT=wasm.$PING_PONG_ADDRESS

          # TODO(aeryz): get the port id and channel info as parameters
          jq \
           --arg cw20_port $CW20_PORT \
           '.app_state.ibc.channel_genesis.channels += [{
              "state": 3,
              "ordering": 1,
              "counterparty": {
                "port_id": "transfer",
                "channel_id": "channel-0"
              },
              "connection_hops": ["connection-0"],
              "version": "ics20-1",
              "port_id": $cw20_port,
              "channel_id": "channel-0"
            }]' \
            $out/config/genesis.json | sponge $out/config/genesis.json

          jq \
           --arg cw20_port $CW20_PORT \
           --arg ping_pong_port $PING_PONG_PORT \
           '.app_state.ibc.channel_genesis.send_sequences += [{
              "port_id": $cw20_port,
              "channel_id": "channel-0",
              "sequence": "1"
            }, {
              "port_id": $ping_pong_port,
              "channel_id": "channel-1",
              "sequence": "1"
            }]' \
            $out/config/genesis.json | sponge $out/config/genesis.json

          jq \
           --arg cw20_port $CW20_PORT \
           --arg ping_pong_port $PING_PONG_PORT \
           '.app_state.ibc.channel_genesis.recv_sequences += [{
              "port_id": $cw20_port,
              "channel_id": "channel-0",
              "sequence": "1"
            }, {
              "port_id": $ping_pong_port,
              "channel_id": "channel-1",
              "sequence": "1"
            }]' \
            $out/config/genesis.json | sponge $out/config/genesis.json

          jq \
           --arg cw20_port $CW20_PORT \
           --arg ping_pong_port $PING_PONG_PORT \
           '.app_state.ibc.channel_genesis.ack_sequences += [{
              "port_id": $cw20_port,
              "channel_id": "channel-0",
              "sequence": "1"
            }, {
              "port_id": $ping_pong_port,
              "channel_id": "channel-1",
              "sequence": "1"
            }]' \
            $out/config/genesis.json | sponge $out/config/genesis.json

          jq \
           --arg ping_pong_port $PING_PONG_PORT \
           '.app_state.ibc.channel_genesis.channels += [{
              "state": 3,
              "ordering": 1,
              "counterparty": {
                "port_id": "ping-pong",
                "channel_id": "channel-1"
              },
              "connection_hops": ["connection-0"],
              "version": "ics20-1",
              "port_id": $ping_pong_port,
              "channel_id": "channel-1"
            }]' \
            $out/config/genesis.json | sponge $out/config/genesis.json

          jq \
            '.app_state.ibc.channel_genesis.next_channel_sequence = "2"' \
            $out/config/genesis.json | sponge $out/config/genesis.json

          jq \
            '.app_state.capability.index = "3"' \
            $out/config/genesis.json | sponge $out/config/genesis.json

          jq \
           --arg capability1 capabilities/ports/$CW20_PORT/channels/channel-0 \
           --arg capability2 capabilities/ports/$PING_PONG_PORT/channels/channel-1 \
            '.app_state.capability.owners += [{
              "index": "1",
              "index_owners": {
                "owners": [
                  {
                    "module": "ibc",
                    "name": $capability1
                  },
                  {
                    "module": "wasm",
                    "name": $capability1
                  }
                ]
              }
            },{
              "index": "2",
              "index_owners": {
                "owners": [
                  {
                    "module": "ibc",
                    "name": $capability2
                  },
                  {
                    "module": "wasm",
                    "name": $capability2
                  }
                ]
              }
            } ]' \
            $out/config/genesis.json | sponge $out/config/genesis.json

        '';

      addLightClientCodeToGenesis = contract: home: pkgs.runCommand "add-light-client-contract-code-to-genesis"
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
          addContract = { contract, idx }: home: pkgs.runCommand "add-ibc-contract-code-to-genesis"
            {
              buildInputs = [ pkgs.jq pkgs.moreutils pkgs.xxd ];
            }
            ''
              export HOME=$(pwd)
              mkdir -p $out
              cp --no-preserve=mode -r ${home}/* $out

              ALICE_ADDRESS=$(${simd} keys list \
                --keyring-backend test \
                --home $out \
                --output json \
                | jq '.[] | select(.name == "${genesisAccountName}").address' --raw-output)

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
          "add-ibc-contract-codes-to-genesis"
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

      mkHome = { genesisAccounts }: home:
        pkgs.runCommand "genesis-home"
          {
            buildInputs = [ pkgs.jq pkgs.moreutils pkgs.expect ];
          }
          ''
            export HOME=$(pwd)
            mkdir -p $out
            cp --no-preserve=mode -r ${home}/* $out

            ${builtins.concatStringsSep "\n" (builtins.map (key: ''
              key_base=${key}
              val_index=''${key_base//[^0-9]/}
              echo $val_index
              echo "
                set timeout 30
                spawn ${simd} keys mnemonic --unsafe-entropy --home $out
                expect \"WARNING:\"
                send \"abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz$val_index\\r\"
                expect \"Input length:\"
                send \"y\\r\"
                expect eof
              " > expect-${key}.exp
              val_mnemonic=$(expect expect-${key}.exp | tail -n 1)
              echo $val_mnemonic

              echo $val_mnemonic | ${simd} keys add --recover ${key} --keyring-backend test --home $out
              ${simd} genesis add-genesis-account ${key} 100000000000000000000000000stake --keyring-backend test --home $out
            '') genesisAccounts)}
          '';
      mkValidatorKeys = { validatorCount, home }:
        let
          knownKeys = [
            ''{"address":"12729FC85FF80E52064B6F46312B77C95F90F4BF","pub_key":{"type":"tendermint/PubKeyEd25519","value":"2tuto808JS1lD9lYm3KhW4o5b+/eISsMvlzIfR3lmL8="},"priv_key":{"type":"tendermint/PrivKeyEd25519","value":"XkKF6ZWhMjoEhs5/okcVt8w8zsFRl6/y4eCs1cX4nHDa262jzTwlLWUP2VibcqFbijlv794hKwy+XMh9HeWYvw=="}}''
            ''{"address":"0217A42A8BEA30521411A8B34BBFBEABF81DAA1D","pub_key":{"type":"tendermint/PubKeyEd25519","value":"xGHJ9mra+rwc09Glf9aetO44QgUKuHN7IaAp324N92g="},"priv_key":{"type":"tendermint/PrivKeyEd25519","value":"4MKI2W8/GujkKIDPPqdkMZoituxeopbap3+I+FgVCsfEYcn2atr6vBzT0aV/1p607jhCBQq4c3shoCnfbg33aA=="}}''
            ''{"address":"55C7594DBA46848C8241BD06E400129A1082CD4C","pub_key":{"type":"tendermint/PubKeyEd25519","value":"BcjjM1+YBIMYP/lIS+JViyIdXMXoHEom09cyafzyR1k="},"priv_key":{"type":"tendermint/PrivKeyEd25519","value":"xVz5eJxZU2ySxCIqdWHMEgHTSzjU6+m9K1EmxN0qy7EFyOMzX5gEgxg/+UhL4lWLIh1cxegcSibT1zJp/PJHWQ=="}}''
            ''{"address":"3FB23E5CD869EE24A00604BCF0B9A2696AB0B599","pub_key":{"type":"tendermint/PubKeyEd25519","value":"KAuqSUd1+wqaozlFuhHVjpxszkUkygpM4jOeU42lrF4="},"priv_key":{"type":"tendermint/PrivKeyEd25519","value":"TI1KG4RD+Zx626D7zahcGeh4Q2wSkrh/f+R1NVaLyJMoC6pJR3X7CpqjOUW6EdWOnGzORSTKCkziM55TjaWsXg=="}}''
            ''{"address":"12729FC85FF80E52064B6F46312B77C95F90F4BF","pub_key":{"type":"tendermint/PubKeyEd25519","value":"2tuto808JS1lD9lYm3KhW4o5b+/eISsMvlzIfR3lmL8="},"priv_key":{"type":"tendermint/PrivKeyEd25519","value":"XkKF6ZWhMjoEhs5/okcVt8w8zsFRl6/y4eCs1cX4nHDa262jzTwlLWUP2VibcqFbijlv794hKwy+XMh9HeWYvw=="}}''
          ];
        in
        builtins.genList
          (i:
            pkgs.runCommand "valkey-${toString i}" { } ''
              mkdir -p $out
              cp ${builtins.toFile "valkey-${toString i}.json" (builtins.elemAt knownKeys i)} $out/valkey-${toString i}.json
            '')
          validatorCount;
      mkValidatorGentx = { home, validatorKeys }:
        pkgs.lib.lists.imap0
          (i: valKey:
            pkgs.runCommand "valgentx-${toString i}"
              {
                buildInputs = [ pkgs.jq ];
              }
              ''
                export HOME=$(pwd)

                PUBKEY=`cat ${valKey}/valkey-${
                  toString i
                }.json | jq ."pub_key"."value"`
                PUBKEY="{\"@type\":\"/cosmos.crypto.ed25519.PubKey\",\"key\":$PUBKEY}"
                mkdir -p $out
                ${simd} genesis gentx val-${toString i} 1000000000000000000000stake --keyring-backend test --chain-id ${chainId} --home ${home} --ip "0.0.0.0" --pubkey $PUBKEY --moniker validator-${toString i} --output-document $out/valgentx-${
                  toString i
                }.json
              '')
          validatorKeys;
      genesisHome = pkgs.lib.foldl
        (home: f: f home)
        initHome
        (
          # add light clients
          (builtins.map addLightClientCodeToGenesis [
            self'.packages.cometbls-light-client
          ])
          ++ [
            add08WasmToAllowedClients

            # add ibc contracts
            (addIbcContractCodesToGenesis [
              self'.packages.ucs01-relay
              self'.packages.ucs00-pingpong
            ])
            # add ibc connection and channel
            addIbcConnectionToGenesis
            addIbcChannelToGenesis

            (mkHome {
              genesisAccounts = builtins.genList (i: "val-${toString i}") devnetConfig.validatorCount;
            })
          ]
        )
      ;
      validatorKeys = mkValidatorKeys { inherit (devnetConfig) validatorCount; home = genesisHome; };
      validatorGentxs = mkValidatorGentx {
        home = genesisHome;
        inherit validatorKeys;
      };
      validatorNodeIDs = { validatorCount }: builtins.genList (i: mkNodeID "valnode-${toString i}.json") validatorCount;
    in
    {
      packages.simd-genesis = pkgs.runCommand "genesis" { } ''
        mkdir $out
        cd $out

        export HOME=$(pwd)

        # Copy the read-only genesis we used to build the genesis file as the collect-gentxs command will overwrite it
        cp --no-preserve=mode -r ${genesisHome}/* .

        mkdir ./config/gentx
        ${builtins.concatStringsSep "\n" (pkgs.lib.lists.imap0 (i: valGentx: ''
          cp ${valGentx}/valgentx-${toString i}.json ./config/gentx/valgentx-${
            toString i
          }.json
        '') validatorGentxs)}

        ${simd} genesis collect-gentxs --home . 2> /dev/null
        ${simd} genesis validate --home .
      '';

      packages.simd-validator-keys = pkgs.symlinkJoin {
        name = "validator-keys";
        paths = validatorKeys;
      };

      packages.simd-validator-gentxs = pkgs.symlinkJoin {
        name = "validator-gentxs";
        paths = validatorGentxs;
      };

      packages.simd-validator-node-ids = pkgs.symlinkJoin {
        name = "validator-node-ids";
        paths = validatorNodeIDs { inherit (devnetConfig) validatorCount; };
      };

      checks = { };
    };
}
