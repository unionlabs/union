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
      uniond = pkgs.lib.getExe self'.packages.uniond;
      chainId = "union-devnet-1";
      mkNodeID = name:
        pkgs.runCommand "node-id" { } ''
          export HOME=$(pwd)

          ${uniond} init testnet bn254 --chain-id ${chainId} --home .
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

          ${uniond} init testnet bn254 --chain-id ${chainId} --home $out

          echo ${alice} | ${uniond} keys add \
            --recover ${genesisAccountName} \
            --keyring-backend test \
            --home $out

          ${uniond} add-genesis-account ${genesisAccountName} 10000000000000000000000000stake \
            --keyring-backend test \
            --home $out

          ${builtins.concatStringsSep "\n" (pkgs.lib.lists.imap0 (i: mnemonic: ''
            # Add the dev account
            echo ${mnemonic} | ${uniond} keys add \
              --recover ${genesisAccountName}-${toString i} \
              --keyring-backend test \
              --home $out
            ${uniond} add-genesis-account ${genesisAccountName}-${toString i} 10000000000000000000000000stake \
              --keyring-backend test \
              --home $out
          '') devMnemonics)}
        '';

      applyGenesisOverwrites = genesisOverwrites: home:
        let
          overwrites = builtins.toFile "overwrite.json" (builtins.toJSON genesisOverwrites);
        in
        pkgs.runCommand "apply-genesis-overwrites"
          {
            buildInputs = [ pkgs.jq ];
          }
          ''
            mkdir -p $out
            cp --no-preserve=mode -r ${home}/* $out
            jq -s '.[0] * .[1]' ${home}/config/genesis.json ${overwrites} > merge.json
            mv merge.json $out/config/genesis.json
          '';

      calculateCw20Ics20ContractAddress = home: pkgs.runCommand "calculate-ucs01-relay-contract-address"
        {
          buildInputs = [ pkgs.jq ];
        }
        ''
          export HOME=$(pwd)
          mkdir -p $out
          cp --no-preserve=mode -r ${home}/* $out

          ALICE_ADDRESS=$(${uniond} keys list \
            --keyring-backend test \
            --home $out \
            --output json \
            | jq '.[] | select(.name == "${genesisAccountName}").address' --raw-output)

          CODE_HASH=$(sha256sum ${self'.packages.ucs01-relay}/lib/ucs01_relay.wasm | cut -f1 -d" ")

          ${uniond} query wasm build-address $CODE_HASH $ALICE_ADDRESS ${cw-instantiate2-salt} > $out/CW20_ICS20_CONTRACT_ADDRESS
        '';

      calculatePingPongAddress = home: pkgs.runCommand "calculate-ping-pong-contract-address"
        {
          buildInputs = [ pkgs.jq ];
        }
        ''
          export HOME=$(pwd)
          mkdir -p $out
          cp --no-preserve=mode -r ${home}/* $out

          ALICE_ADDRESS=$(${uniond} keys list \
            --keyring-backend test \
            --home $out \
            --output json \
            | jq '.[] | select(.name == "${genesisAccountName}").address' --raw-output)

          CODE_HASH=$(sha256sum ${self'.packages.ucs00-pingpong}/lib/ucs00_pingpong.wasm | cut -f1 -d" ")

          ${uniond} query wasm build-address $CODE_HASH $ALICE_ADDRESS ${cw-instantiate2-salt} > $out/PING_PONG_CONTRACT_ADDRESS
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

          ALICE_ADDRESS=$(${uniond} keys list \
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

              ALICE_ADDRESS=$(${uniond} keys list \
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
                spawn ${uniond} keys mnemonic --unsafe-entropy --home $out
                expect \"WARNING:\"
                send \"abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz$val_index\\r\"
                expect \"Input length:\"
                send \"y\\r\"
                expect eof
              " > expect-${key}.exp
              val_mnemonic=$(expect expect-${key}.exp | tail -n 1)
              echo $val_mnemonic

              echo $val_mnemonic | ${uniond} keys add --recover ${key} --keyring-backend test --home $out
              ${uniond} add-genesis-account ${key} 100000000000000000000000000stake --keyring-backend test --home $out
            '') genesisAccounts)}
          '';
      mkValidatorKeys = { validatorCount, home }:
        let
          knownKeys = [
            ''{"address":"EFB1D8B3A56D97F2AB24AC5F0B04F48535F74DA9","pub_key":{"type":"tendermint/PubKeyBn254","value":"ht8ttsjmD9S+0ZQKLjKp9iUSnhOlFWAjqfGDnoCjHfg="},"priv_key":{"type":"tendermint/PrivKeyBn254","value":"5HSpb7qsbzmIZKZJ97NaaqXsP0EjG7ddmHRezrdZJFEbCVyh1VhArkenyrEFwa+NNaG6x1EKSbrZ/5No/IDs6A=="}}''
            ''{"address":"4CE57693C82B50F830731DAB14FA759327762456","pub_key":{"type":"tendermint/PubKeyBn254","value":"7ZAoR4jcMmiqojusF0tkv/Q27wYPXAVieQWEzvUsW9g="},"priv_key":{"type":"tendermint/PrivKeyBn254","value":"wyOxb9YgCWVB2Z/y5xOECtpDb6rZIzGn5ohx3CZDM/4NwR+HcK/aRlazPAGn3+HKvuwZb7XP5+wrOzhGKTiYVA=="}}''
            ''{"address":"36E1644D94064ED11521041E9138A0D1CCA9C31C","pub_key":{"type":"tendermint/PubKeyBn254","value":"jZiv55ih+4mChYy+Jm3M/u/MA5ZK530uMkgqgBcQnfo="},"priv_key":{"type":"tendermint/PrivKeyBn254","value":"jnw+EPjkwoXGXSzBhYQXX+SXxDH+l9AwD+YkZ1eSRj4qP6SCyDxr75CmldLiqdCfl62ld12XiYrER04rVgunqg=="}}''
            ''{"address":"196D6009588DA28CF40039C957A53B08104723F9","pub_key":{"type":"tendermint/PubKeyBn254","value":"k/tDqzvtGyDwEI6mUX9qpL+pbP+GeYPpZC5XQiSU12Q="},"priv_key":{"type":"tendermint/PrivKeyBn254","value":"nOVOW+JEBz4zv4ffzIfRg2FE3iq95chGCjvZ99n6Y5cRI3XH08xMGSW8BH416Swp+oU25fWMeRRnqaMCbaW4Fw=="}}''
            ''{"address":"C5AFE5C76192ACD502AB9D9D88CBC9C75597C411","pub_key":{"type":"tendermint/PubKeyBn254","value":"nI931rYm57np2qqZLxwGLZYQkrXiMUPckaxneyZss98="},"priv_key":{"type":"tendermint/PrivKeyBn254","value":"7tKq4oTOHRxIZTiAJhUmlt3dSSuAaFeLmr2gEOWp3OMYdIiXiCk0FGPcsqT0m5ETpr0i9yqq02gcjpg7F4Yd1A=="}}''
            ''{"address":"2DCE4E05E127F97B23F8099E4D1DBDEB7587DC8B","pub_key":{"type":"tendermint/PubKeyBn254","value":"mMjsEy9PZLJLGURHF1KXRlpgdS38eCbztA/wYUUuO+w="},"priv_key":{"type":"tendermint/PrivKeyBn254","value":"3+Xec6VtBROuaEjTm6iv2t6gFdfNPaSdK/0L+Qv0a40BIY55SXbyzEOvXa/FZrXI4LhoTpX3g1Gg72O/lWrIhg=="}}''
            ''{"address":"19963640A11B2EC4F08E5B5000CD30D8641AA569","pub_key":{"type":"tendermint/PubKeyBn254","value":"p7jGEk8mMgsCp1KPonEoJoo48AHxIj7csAU61OlEEhs="},"priv_key":{"type":"tendermint/PrivKeyBn254","value":"2t30BrZyq0nKp3DDUaasR7KyI8etiZw/Jp7hOHlpNssB/VEtzcUckBRimwwYbUFM3x1z4LwuRKDKOLxt0M1kRA=="}}''
            ''{"address":"F3A5615BEB78B0D297FE37254433D7C0C367158A","pub_key":{"type":"tendermint/PubKeyBn254","value":"rW4uEup6ZPtH6RHeCBltigC7P6y+mTF0XSkAu8zfXnk="},"priv_key":{"type":"tendermint/PrivKeyBn254","value":"iyZVrpbqgM+Yg6a01BdK4NxKUMRg4oK7AE6zj05wlNcwNJ2mRfhznKxx9CzKgrx6+v8fnhTuTPlocFyQM+I4EA=="}}''
            ''{"address":"0C26B59A0B65D191A86D969D5D3F2DC40DD9C977","pub_key":{"type":"tendermint/PubKeyBn254","value":"wUIdx4VSAyjBSD7KGxEHhE19IczlZFFEmNFf2dIzklM="},"priv_key":{"type":"tendermint/PrivKeyBn254","value":"zCiKDLoQcJ62QFmDV0T9tQ9mMtqRAxKFFBQLItg+EtEgAkNRg+C5yYTJnUD3kDTcHoZhXQGb1pXls5jyrO363Q=="}}''
            ''{"address":"D619CD0E08ED87F92BB7BC2171071CFAC7BE1A4B","pub_key":{"type":"tendermint/PubKeyBn254","value":"wV8abg4Z83e0/NFv8E2yoj07lzSmxZGsHfi7NkEbKX0="},"priv_key":{"type":"tendermint/PrivKeyBn254","value":"y7W98lZYXbR59QqpKpsl7+u0Z1CO20f554+QUR1IQjYgP3HdDnfPZ6+LPE3UsR9N5y+Uo7upPF+Az1keRy3MGg=="}}''
            ''{"address":"257BC7E3F7BAD2C5EB1A11318003FE6CB5A52BE5","pub_key":{"type":"tendermint/PubKeyBn254","value":"gAM5W+LDW4eFkZw2n3WDmCCd565WTDd5E7L2L7yzOW8="},"priv_key":{"type":"tendermint/PrivKeyBn254","value":"lwGosINoR6TB7RxYbPSmsDkP/s9dIA188I8GDylWHSABm8FveFaSttwExzXtHPBpyJ1VJzo6Iti5RsF9uBDmNw=="}}''
            ''{"address":"00B978986867D21B0A93DACD62A7EDD3D913F3D9","pub_key":{"type":"tendermint/PubKeyBn254","value":"hHtgMOdYMU8muqxX5PrdjYWRsIZ9cwezbE2gz5vVFpo="},"priv_key":{"type":"tendermint/PrivKeyBn254","value":"qvf266JEKyEkXA7ynj19jZ7hAXVJIoFI4W+2uekTIh8WXXf7cqhh6CHddn9ceZayA9fte7K4nqladzVMeztBmA=="}}''
            ''{"address":"245DC189905D4F57D26EAE5120377707ED56ECA0","pub_key":{"type":"tendermint/PubKeyBn254","value":"ley/CHKtnWvu5aVxbfU9jgcWRkWV+j2bSmYNqgK8nAY="},"priv_key":{"type":"tendermint/PrivKeyBn254","value":"h10M+nsocY8s+V5N4SOVuNkTdWhDFr1vVy1PE8q5tZEbrHPSZ0oMhSATpQQVuJioawJNvwSl9qz8HsQLZeLdDQ=="}}''
            ''{"address":"DFA8398671155E09BBA8244C2D7C295F980F4A2A","pub_key":{"type":"tendermint/PubKeyBn254","value":"4KRIncS9hK37sD0cHGDcFI0EEu8T7I/JFEiGKVefx3U="},"priv_key":{"type":"tendermint/PrivKeyBn254","value":"orJ8sHmfUeCpT52USoWDGcG0ggV20QmhxE/Ni8zVEScKT0N0Qj0KqgX5WvLyeEojuPUHrqiLwdnYQvf8dgxC/g=="}}''
            ''{"address":"503662ECE25CF73487F100EDD02D775EEFEFCD0E","pub_key":{"type":"tendermint/PubKeyBn254","value":"mUhqAu7NxIjEALZXq2X0jeYW0wTcaccgsrnxbusUKCQ="},"priv_key":{"type":"tendermint/PrivKeyBn254","value":"6XbI5tf4iFBqgN0qJ/9ndOIgTGM7nzh7PYYIGLAbd2EDu/PKsKmkcDqNxo9Vq5VuJ6iLp7AUJmd5PqbbdGKNnQ=="}}''
            ''{"address":"94FEEE87198F3AD180733D7B3C10FD2F150C3E62","pub_key":{"type":"tendermint/PubKeyBn254","value":"j2f0mRA51Iz9VQNu131t/7V0a4k19azWsyiimmUPkoU="},"priv_key":{"type":"tendermint/PrivKeyBn254","value":"2z+ZOx4A3JtdwsUHyAApPw6nxjbPwttXtAYSJK8Mfy8He7ZHgc9BVX0bKke6AnQdpUGUqe2ar9yG8VoTn/EFuQ=="}}''
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
                PUBKEY="{\"@type\":\"/cosmos.crypto.bn254.PubKey\",\"key\":$PUBKEY}"
                mkdir -p $out
                ${uniond} genesis gentx val-${toString i} 1000000000000000000000stake "bn254" --keyring-backend test --chain-id ${chainId} --home ${home} --ip "0.0.0.0" --pubkey $PUBKEY --moniker validator-${toString i} --output-document $out/valgentx-${
                  toString i
                }.json
              '')
          validatorKeys;
      genesisHome = pkgs.lib.foldl
        (home: f: f home)
        initHome
        (
          [ (applyGenesisOverwrites devnetConfig.genesisOverwrites) ]
          # add light clients
          ++ (builtins.map addLightClientCodeToGenesis [
            self'.packages.ethereum-light-client-minimal
            self'.packages.ethereum-light-client-mainnet
          ])
          # add ibc contracts
          ++ [
            add08WasmToAllowedClients

            (addIbcContractCodesToGenesis [
              self'.packages.ucs01-relay
              self'.packages.ucs00-pingpong
            ])
          ]
          # add ibc connection
          ++ [
            (addIbcConnectionToGenesis)
          ]
          ++ [
            (addIbcChannelToGenesis)
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
      packages.devnet-genesis = pkgs.runCommand "genesis" { } ''
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

        ${uniond} genesis collect-gentxs --home . 2> /dev/null
        ${uniond} genesis validate --home .
      '';

      packages.devnet-validator-keys = pkgs.symlinkJoin {
        name = "validator-keys";
        paths = validatorKeys;
      };

      packages.devnet-validator-gentxs = pkgs.symlinkJoin {
        name = "validator-gentxs";
        paths = validatorGentxs;
      };

      packages.devnet-validator-node-ids = pkgs.symlinkJoin {
        name = "validator-node-ids";
        paths = validatorNodeIDs { inherit (devnetConfig) validatorCount; };
      };

      packages.devnet-eth-config = pkgs.linkFarm "devnet-eth-config" [
        { name = "genesis.json"; path = "${./devnet-eth/genesis.json}"; }
        { name = "dev-key0.prv"; path = "${./devnet-eth/dev-key0.prv}"; }
        { name = "dev-key1.prv"; path = "${./devnet-eth/dev-key1.prv}"; }
        { name = "dev-jwt.prv"; path = "${./devnet-eth/dev-jwt.prv}"; }
      ];

      checks = { };
    };
}
