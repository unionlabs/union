{ ... }: {
  perSystem = { devnetConfig, pkgs, self', ... }:
    let
      uniond = pkgs.lib.getExe self'.packages.uniond;
      chainId = "union-devnet-1";
      mkNodeID = name:
        pkgs.runCommand "node-id" { } ''
          ${uniond} init testnet bn254 --chain-id ${chainId} --home .
          mkdir -p $out
          mv ./config/node_key.json $out/${name}
        '';
      mkClientState = 
      mkHome = { genesisAccounts }:
        pkgs.runCommand "genesis-home" { } ''
          mkdir -p $out
          
          # Generate the wasm client genesis state
          base64 -w0 ${self'.packages.wasm-ethereum-lc}/lib/union_ethereum_lc.wasm > $out/encoded.txt
          ${uniond} init testnet bn254 --chain-id ${chainId} --home $out
          CHECKSUM=`sha256sum ${self'.packages.wasm-ethereum-lc}/lib/union_ethereum_lc.wasm | cut -f1 -d " "`
          CODE_ID=`echo -ne codeId/$CHECKSUM | base64 -w0`
          # CODE_ID_B64=`${pkgs.python3}/bin/python3 -c "import base64;import codecs;bytesObj=codecs.decode(\"$CHECKSUM\",'hex_codec');print(base64.b64encode(bytesObj).decode())"`
          cat $out/config/genesis.json | \
             ${pkgs.jq}/bin/jq --arg code_id $CODE_ID --rawfile encoded_file $out/encoded.txt '.app_state."08-wasm".contracts'='[ { "code_id_key": $code_id, "contract_code": $encoded_file }  ]' \
          > $out/tmp-genesis.json
          # | ${pkgs.jq}/bin/jq --arg code_id $CODE_ID_B64 .app_state.ibc.client_genesis.clients='[{"client_id":"08-wasm-0","client_state":{"@type":"/ibc.lightclients.wasm.v1.ClientState","code_id":$code_id,"data":"e30=","latest_height":{"revision_height":"2","revision_number":"0"}}}]' | ${pkgs.jq}/bin/jq .app_state.ibc.client_genesis.clients_consensus='[{"client_id":"08-wasm-0","consensus_states":[{"consensus_state":{"@type":"/ibc.lightclients.wasm.v1.ConsensusState","data":"e30=","timestamp":"1678732260023000000"},"height":{"revision_height":"1","revision_number":"0"}}]}]' \
          mv $out/tmp-genesis.json $out/config/genesis.json
            
          # Add the dev account
          ADDRESS=`echo 'wine parrot nominee girl exchange element pudding grow area twenty next junior come render shadow evidence sentence start rough debate feed all limb real' | ${uniond} keys add --recover testkey --keyring-backend test --home $out --output json | ${pkgs.jq}/bin/jq -r .address`
          ${uniond} add-genesis-account $ADDRESS 10000000000000000000000000stake --keyring-backend test --home $out

          ${builtins.concatStringsSep "\n" (builtins.map (key: ''
            ${uniond} keys add ${key} --keyring-backend test --home $out
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

        ${uniond} collect-gentxs --home .
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

      packages.devnet-geth-config = pkgs.linkFarm "devnet-geth-config" [
        { name = "genesis.json"; path = "${./devnet-evm/genesis.json}"; }
        { name = "dev-key0.prv"; path = "${./devnet-evm/dev-key0.prv}"; }
        { name = "dev-key1.prv"; path = "${./devnet-evm/dev-key1.prv}"; }
        { name = "dev-jwt.prv"; path = "${./devnet-evm/dev-jwt.prv}"; }
      ];

      packages.devnet-lodestar-config = pkgs.linkFarm "lodestar-config" [
        { name = "dev-jwt.prv"; path = "${./devnet-evm/dev-jwt.prv}"; }
      ];
      checks = { };
    };
}
