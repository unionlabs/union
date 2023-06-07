#cspell:ignore abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz
{ ... }: {
  perSystem = { devnetConfig, system, pkgs, self', inputs', ... }:
    let
      uniond = pkgs.lib.getExe self'.packages.uniond;
      chainId = "union-devnet-1";
      mkNodeID = name:
        pkgs.runCommand "node-id" { } ''
          ${uniond} init testnet bn254 --chain-id ${chainId} --home .
          mkdir -p $out
          mv ./config/node_key.json $out/${name}
        '';
      mkHome = { genesisAccounts }:
        pkgs.runCommand "genesis-home" { } ''
          export HOME=$(pwd)
          mkdir -p $out

          # Generate the wasm client genesis state
          base64 -w0 ${self'.packages.wasm-ethereum-lc}/lib/union_ethereum_lc.wasm > $out/encoded.txt
          ${uniond} init testnet bn254 --chain-id ${chainId} --home $out
          CHECKSUM=`sha256sum ${self'.packages.wasm-ethereum-lc}/lib/union_ethereum_lc.wasm | cut -f1 -d " "`
          CODE_ID=`echo -ne codeId/$CHECKSUM | base64 -w0`

          cat $out/config/genesis.json | \
             ${pkgs.jq}/bin/jq --arg code_id $CODE_ID --rawfile encoded_file $out/encoded.txt '.app_state."08-wasm".contracts'='[ { "code_id_key": $code_id, "contract_code": $encoded_file }  ]' \
          > $out/tmp-genesis.json

          mv $out/tmp-genesis.json $out/config/genesis.json

          # Add the dev account
          ADDRESS=`echo 'wine parrot nominee girl exchange element pudding grow area twenty next junior come render shadow evidence sentence start rough debate feed all limb real' | ${uniond} keys add --recover testkey --keyring-backend test --home $out --output json | ${pkgs.jq}/bin/jq -r .address`
          ${uniond} add-genesis-account $ADDRESS 10000000000000000000000000stake --keyring-backend test --home $out

          # Store and instantiate cw20-ics20 code in genesis
          base64 -w0 ${self'.packages.wasm-cw20-ics20}/lib/cw20_ics20.wasm > $out/encoded_cw20_ics20.txt
          # Hex hash of cw20-ics20 contract
          CW20_ICS20_CODE_HASH=`sha256sum ${self'.packages.wasm-cw20-ics20}/lib/cw20_ics20.wasm | cut -f1 -d" "`
          # Base64 hash of cw20-ics20 contract
          CW20_ICS20_CODE_HASH_B64=`echo $CW20_ICS20_CODE_HASH | ${pkgs.xxd}/bin/xxd -r -p | base64`
          # Address that is calculated from the hex hash, this will be used as the contract address
          CW20_ICS20_ADDRESS=`${uniond} keys parse $CW20_ICS20_CODE_HASH --output json | ${pkgs.jq}/bin/jq '.formats[0]' -r`
          # Last code id key, this will be number of stored codes + 1, it needs to be prefixed with 0x4
          LAST_CODE_ID_KEY=`echo -ne "\x04lastCodeId" | base64`
          # Last contract id key, this will be number of instantiated contracts + 1, it needs to be prefixed with 0x4
          LAST_CONTRACT_ID_KEY=`echo -ne "\x04lastContractId" | base64`

          # NOTE(aeryz): `contract_state` is the state of the storage which is queried from the running chain. It's hard to generate programmatically.

          cat $out/config/genesis.json | \
          ${pkgs.jq}/bin/jq --arg creator $ADDRESS --arg code_hash $CW20_ICS20_CODE_HASH_B64 --rawfile cw20_ics20_encoded $out/encoded_cw20_ics20.txt '.app_state.wasm.codes'='[ 
            {
              "code_id": "1", 
              "code_info": { 
                "code_hash": $code_hash, 
                "creator": $creator, 
                "instantiate_config": { "permission": "Everybody" } 
              }, 
              "code_bytes": $cw20_ics20_encoded, 
              "pinned": false 
            }]' | \
          ${pkgs.jq}/bin/jq --arg address $CW20_ICS20_ADDRESS --arg creator $ADDRESS '.app_state.wasm.contracts'='[
            {
              "contract_address": $address,
              "contract_info": { 
                "code_id":  1, 
                "creator": $creator, 
                "admin": $creator, 
                "label": "cw20-ics20-test", 
                "created": { "block_height": 0, "tx_index": 0 } 
              }, 
              "contract_state": [ 
                {"key":"61646D696E","value":"InVuaW9uMWprOXBzeWh2Z2tydDJjdW16OGV5dGxsMjI0NG0ybm56NHl0MmcyIg=="}, 
                {"key":"636F6E74726163745F696E666F","value":"eyJjb250cmFjdCI6ImNyYXRlcy5pbzpjdzIwLWljczIwIiwidmVyc2lvbiI6IjEuMC4xIn0="}, 
                {"key":"69637332305F636F6E666967","value":"eyJkZWZhdWx0X3RpbWVvdXQiOjk5OTk5OTk5LCJkZWZhdWx0X2dhc19saW1pdCI6bnVsbH0="} 
              ],  
              "contract_code_history": [ 
                { 
                  "operation": "CONTRACT_CODE_HISTORY_OPERATION_TYPE_INIT", 
                  "code_id": "1", 
                  "updated":  {"block_height": "0", "tx_index": "0" }, 
                  "msg": { "default_timeout": 99999999, "gov_contract": "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2", "allowlist": [] } 
                } 
              ] 
            }]' | \
            ${pkgs.jq}/bin/jq --arg last_contract_id_key $LAST_CONTRACT_ID_KEY --arg last_code_id_key $LAST_CODE_ID_KEY '.app_state.wasm.sequences'='[ 
              { "id_key": $last_code_id_key, "value": 2 }, 
              { "id_key": $last_contract_id_key, "value": 2 }
            ]' \
          > $out/tmp-genesis.json
          
          mv $out/tmp-genesis.json $out/config/genesis.json


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
            val_mnemonic=$(${pkgs.expect}/bin/expect expect-${key}.exp | tail -n 1)
            echo $val_mnemonic

            echo $val_mnemonic | ${uniond} keys add --recover ${key} --keyring-backend test --home $out
            ${uniond} add-genesis-account ${key} 100000000000000000000000000stake --keyring-backend test --home $out
          '') genesisAccounts)}
        '';
      mkValidatorKeys = { validatorCount, home }:
        builtins.genList
          (i:
            pkgs.runCommand "valkey-${toString i}" { } ''
              mkdir -p $out
              ${uniond} genbn --home ${home} >> $out/valkey-${toString i}.json
            '')
          validatorCount;
      mkValidatorGentx = { home, validatorKeys }:
        pkgs.lib.lists.imap0
          (i: valKey:
            pkgs.runCommand "valgentx-${toString i}" { } ''
              PUBKEY=`cat ${valKey}/valkey-${
                toString i
              }.json | ${pkgs.jq}/bin/jq ."pub_key"."value"`
              PUBKEY="{\"@type\":\"/cosmos.crypto.bn254.PubKey\",\"key\":$PUBKEY}"
              mkdir -p $out
              ${uniond} gentx val-${toString i} 1000000000000000000000stake "bn254" --keyring-backend test --chain-id ${chainId} --home ${home} --ip "0.0.0.0" --pubkey $PUBKEY --moniker validator-${toString i} --output-document $out/valgentx-${
                toString i
              }.json
            '')
          validatorKeys;
      genesisHome = mkHome {
        genesisAccounts = builtins.genList (i: "val-${toString i}") devnetConfig.validatorCount;
      };
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
