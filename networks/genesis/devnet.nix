{ ... }: {
  perSystem = { devnetConfig, system, pkgs, self', inputs', ... }:
    let
      uniond = pkgs.lib.getExe self'.packages.uniond;
      prysmctl = self'.packages.prysmctl;
      chainId = "union-devnet-1";
      mkNodeID = name:
        pkgs.runCommand "node-id" { } ''
          ${uniond} init testnet bn254 --chain-id ${chainId} --home .
          mkdir -p $out
          mv ./config/node_key.json $out/${name}
        '';
      mkHome = { genesisAccounts }:
        pkgs.runCommand "genesis-home" { } ''
          mkdir -p $out
          ${uniond} init testnet bn254 --chain-id ${chainId} --home $out
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
      mkGethPrysmGenesis = {}:
        pkgs.runCommand "geth-prysm-genesis" { } ''
          mkdir -p $out
          cp ${./devnet-evm/genesis.json} "./genesis.json"
        
          ${prysmctl}/bin/prysmctl \
          testnet generate-genesis \
            --fork=bellatrix \
            --num-validators=64 \
            --output-ssz=$out/genesis.ssz \
            --chain-config-file=${./devnet-evm/beacon-config.yml} \
            --geth-genesis-json-in=./genesis.json \
            --geth-genesis-json-out=$out/genesis.json \
        '';
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

      packages.devnet-geth-prysm-genesis = pkgs.symlinkJoin {
        name = "geth-prysm-genesis-state";
        paths = mkGethPrysmGenesis { };
      };

      packages.devnet-geth-config = pkgs.linkFarm "devnet-geth-config" [
        { name = "genesis.json"; path = "${./devnet-evm/genesis.json}"; }
        { name = "beacon-config.yml"; path = "${./devnet-evm/beacon-config.yml}"; }
        { name = "dev-key0.prv"; path = "${./devnet-evm/dev-key0.prv}"; }
        { name = "dev-key1.prv"; path = "${./devnet-evm/dev-key1.prv}"; }
        { name = "dev-jwt.prv"; path = "${./devnet-evm/dev-jwt.prv}"; }
      ];

      packages.devnet-prysm-config = pkgs.linkFarm "prysm-config" [
        { name = "dev-jwt.prv"; path = "${./devnet-evm/dev-jwt.prv}"; }
        { name = "beacon-config.yml"; path = "${./devnet-evm/beacon-config.yml}"; }
      ];
      checks = { };
    };
}
