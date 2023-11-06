{ ... }: {
  perSystem = { devnetConfig, system, pkgs, self', inputs', cw-instantiate2-salt, ... }:
    let
      MNEMONIC = "wine parrot nominee girl exchange element pudding grow area twenty next junior come render shadow evidence sentence start rough debate feed all limb real";
      genesisAccountName = "testkey";
      wasmd = pkgs.lib.getExe self'.packages.wasmd;
      chainId = "wasm-devnet-1";
      mkNodeID = name:
        pkgs.runCommand "node-id" { } ''
          ${wasmd} init testnet --chain-id ${chainId} --home .
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
          ${wasmd} init testnet --chain-id ${chainId} --home $out
          # Add the dev account
          echo ${MNEMONIC} | ${wasmd} keys add \
            --recover ${genesisAccountName} \
            --keyring-backend test \
            --home $out
          ${wasmd} genesis add-genesis-account ${genesisAccountName} 10000000000000000000000000stake \
            --keyring-backend test \
            --home $out
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
              ALICE_ADDRESS=$(${wasmd} keys list \
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
                spawn ${wasmd} keys mnemonic --unsafe-entropy --home $out
                expect \"WARNING:\"
                send \"abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz$val_index\\r\"
                expect \"Input length:\"
                send \"y\\r\"
                expect eof
              " > expect-${key}.exp
              val_mnemonic=$(expect expect-${key}.exp | tail -n 1)
              echo $val_mnemonic
              echo $val_mnemonic | ${wasmd} keys add --recover ${key} --keyring-backend test --home $out
              ${wasmd} genesis add-genesis-account ${key} 100000000000000000000000000stake --keyring-backend test --home $out
            '') genesisAccounts)}
          '';
      mkValidatorKeys = { validatorCount, home }:
        let
          knownKeys = [
            ''{"address":"12729FC85FF80E52064B6F46312B77C95F90F4BF","pub_key":{"type":"tendermint/PubKeyEd25519","value":"2tuto808JS1lD9lYm3KhW4o5b+/eISsMvlzIfR3lmL8="},"priv_key":{"type":"tendermint/PrivKeyEd25519","value":"XkKF6ZWhMjoEhs5/okcVt8w8zsFRl6/y4eCs1cX4nHDa262jzTwlLWUP2VibcqFbijlv794hKwy+XMh9HeWYvw=="}}''
            ''{"address":"0217A42A8BEA30521411A8B34BBFBEABF81DAA1D","pub_key":{"type":"tendermint/PubKeyEd25519","value":"xGHJ9mra+rwc09Glf9aetO44QgUKuHN7IaAp324N92g="},"priv_key":{"type":"tendermint/PrivKeyEd25519","value":"4MKI2W8/GujkKIDPPqdkMZoituxeopbap3+I+FgVCsfEYcn2atr6vBzT0aV/1p607jhCBQq4c3shoCnfbg33aA=="}}''
            ''{"address":"55C7594DBA46848C8241BD06E400129A1082CD4C","pub_key":{"type":"tendermint/PubKeyEd25519","value":"BcjjM1+YBIMYP/lIS+JViyIdXMXoHEom09cyafzyR1k="},"priv_key":{"type":"tendermint/PrivKeyEd25519","value":"xVz5eJxZU2ySxCIqdWHMEgHTSzjU6+m9K1EmxN0qy7EFyOMzX5gEgxg/+UhL4lWLIh1cxegcSibT1zJp/PJHWQ=="}}''
            ''{"address":"3FB23E5CD869EE24A00604BCF0B9A2696AB0B599","pub_key":{"type":"tendermint/PubKeyEd25519","value":"KAuqSUd1+wqaozlFuhHVjpxszkUkygpM4jOeU42lrF4="},"priv_key":{"type":"tendermint/PrivKeyEd25519","value":"TI1KG4RD+Zx626D7zahcGeh4Q2wSkrh/f+R1NVaLyJMoC6pJR3X7CpqjOUW6EdWOnGzORSTKCkziM55TjaWsXg=="}}''
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
                PUBKEY="{\"@type\":\"/cosmos.crypto.ed25519.PubKey\",\"key\":$PUBKEY}"
                mkdir -p $out
                ${wasmd} genesis gentx val-${toString i} 1000000000000000000000stake --keyring-backend test --chain-id ${chainId} --home ${home} --ip "0.0.0.0" --pubkey $PUBKEY --moniker validator-${toString i} --output-document $out/valgentx-${
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
          # add ibc contracts
          ++ [
            (addIbcContractCodesToGenesis [
              self'.packages.ucs01-relay
              self'.packages.ucs00-pingpong
            ])
          ]
          ++ [
            (mkHome {
              genesisAccounts = builtins.genList (i: "val-${toString i}") 4;
            })
          ]
        )
      ;
      validatorKeys = mkValidatorKeys { validatorCount = 4; home = genesisHome; };
      validatorGentxs = mkValidatorGentx {
        home = genesisHome;
        inherit validatorKeys;
      };
      validatorNodeIDs = { validatorCount }: builtins.genList (i: mkNodeID "valnode-${toString i}.json") validatorCount;
    in
    {
      packages.wasmd-genesis = pkgs.runCommand "genesis" { } ''
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
        ${wasmd} genesis collect-gentxs --home . 2> /dev/null
        ${wasmd} genesis validate-genesis --home .
      '';

      packages.wasmd-validator-keys = pkgs.symlinkJoin {
        name = "validator-keys";
        paths = validatorKeys;
      };

      packages.wasmd-validator-gentxs = pkgs.symlinkJoin {
        name = "validator-gentxs";
        paths = validatorGentxs;
      };

      packages.wasmd-validator-node-ids = pkgs.symlinkJoin {
        name = "validator-node-ids";
        paths = validatorNodeIDs { validatorCount = 4; };
      };

      checks = { };
    };
}