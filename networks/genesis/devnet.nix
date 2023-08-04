#cspell:ignore abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz
{ ... }: {
  perSystem = { devnetConfig, system, pkgs, self', inputs', ... }:
    let
      MNEMONIC = "wine parrot nominee girl exchange element pudding grow area twenty next junior come render shadow evidence sentence start rough debate feed all limb real";
      genesisAccountName = "testkey";
      uniond = pkgs.lib.getExe self'.packages.uniond;
      chainId = "union-devnet-1";
      mkNodeID = name:
        pkgs.runCommand "node-id" { } ''
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

          # Add the dev account
          echo ${MNEMONIC} | ${uniond} keys add \
            --recover ${genesisAccountName} \
            --keyring-backend test \
            --home $out

          ${uniond} add-genesis-account ${genesisAccountName} 10000000000000000000000000stake \
            --keyring-backend test \
            --home $out
        '';

      calculateCw20Ics20ContractAddress = home: pkgs.runCommand "calculate-cw20-ics20-contract-address"
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

          CODE_HASH=$(sha256sum ${self'.packages.wasm-cw20-ics20}/lib/cw20_ics20.wasm | cut -f1 -d" ")

          ${uniond} query wasm build-address $CODE_HASH $ALICE_ADDRESS 61616161 > $out/CW20_ICS20_CONTRACT_ADDRESS
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

          CODE_HASH=$(sha256sum ${self'.packages.wasm-ucs00-pingpong}/lib/ucs00_pingpong.wasm | cut -f1 -d" ")

          ${uniond} query wasm build-address $CODE_HASH $ALICE_ADDRESS 61616161 > $out/PING_PONG_CONTRACT_ADDRESS
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
              "sequence": 1
            }, {
              "port_id": $ping_pong_port,
              "channel_id": "channel-1",
              "sequence": 1
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
            CODE_ID=$(echo -ne "codeId/$CHECKSUM" | base64 -w0)

            echo "code id is '$CODE_ID'"

            mkdir -p $out/code-ids
            echo "$CHECKSUM" > "$out/code-ids/$(basename $wasm .wasm)"

             jq \
              --arg code_id $CODE_ID \
              --rawfile encoded_file encoded \
              '.app_state."08-wasm".contracts += [{
                "code_id_key": $code_id,
                "contract_code": $encoded_file
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
                    "code_id": ${toString idx}, 
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
                  "value": ${toString ((builtins.length contracts) + 1)},
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
                PUBKEY=`cat ${valKey}/valkey-${
                  toString i
                }.json | jq ."pub_key"."value"`
                PUBKEY="{\"@type\":\"/cosmos.crypto.bn254.PubKey\",\"key\":$PUBKEY}"
                mkdir -p $out
                ${uniond} gentx val-${toString i} 1000000000000000000000stake "bn254" --keyring-backend test --chain-id ${chainId} --home ${home} --ip "0.0.0.0" --pubkey $PUBKEY --moniker validator-${toString i} --output-document $out/valgentx-${
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
            self'.packages.ethereum-light-client-minimal
            self'.packages.ethereum-light-client-mainnet
          ])
          # add ibc contracts
          ++ [
            (addIbcContractCodesToGenesis [
              self'.packages.wasm-cw20-ics20
              self'.packages.wasm-ucs00-pingpong
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

        ${uniond} collect-gentxs --home . 2> /dev/null
        ${uniond} validate-genesis --home .
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

      packages.devnet-evm-config = pkgs.linkFarm "devnet-evm-config" [
        { name = "genesis.json"; path = "${./devnet-evm/genesis.json}"; }
        { name = "dev-key0.prv"; path = "${./devnet-evm/dev-key0.prv}"; }
        { name = "dev-key1.prv"; path = "${./devnet-evm/dev-key1.prv}"; }
        { name = "dev-jwt.prv"; path = "${./devnet-evm/dev-jwt.prv}"; }
      ];

      checks = { };
    };
}
