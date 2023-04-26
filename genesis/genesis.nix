{ ... }: {
  perSystem = { pkgs, self', ... }:
    let
      uniond = pkgs.lib.getExe self'.packages.uniond;
      chainId = "union-devnet-1";
      N = 1;
      mkHome = { genesisAccounts }:
        pkgs.runCommand "genesis-home" { } ''
          mkdir -p $out
          ${uniond} init testnet bn254 --chain-id ${chainId} --home $out
          ${builtins.concatStringsSep "\n" (builtins.map (key: ''
            ${uniond} keys add ${key} --keyring-backend test --home $out
            ${uniond} add-genesis-account ${key} 100000000000000000000000000stake --keyring-backend test --home $out
          '') genesisAccounts)}
        '';
      mkValidatorKeys = { home }:
        builtins.genList
          (i:
            pkgs.runCommand "valkey-${toString i}" { } ''
              mkdir -p $out
              ${uniond} genbn --home ${home} >> $out/valkey-${toString i}.json
            '')
          N;
      mkValidatorGentx = { home, txAccount, validatorKeys }:
        pkgs.lib.lists.imap0
          (i: valKey:
            pkgs.runCommand "valgentx-${toString i}" { } ''
              PUBKEY=`cat ${valKey}/valkey-${
                toString i
              }.json | ${pkgs.jq}/bin/jq ."pub_key"."value"`
              PUBKEY="{\"@type\":\"/cosmos.crypto.bn254.PubKey\",\"key\":$PUBKEY}"
              mkdir -p $out
              ${uniond} gentx ${txAccount} 1000000000000000000000stake "bn254" --keyring-backend test --chain-id ${chainId} --home ${home} --ip "0.0.0.0" --pubkey $PUBKEY --sequence ${toString i} --output-document $out/valgentx-${
                toString i
              }.json
            '')
          validatorKeys;
      genesisHome = mkHome { genesisAccounts = [ "alice" ]; };
      validatorKeys = mkValidatorKeys { home = genesisHome; };
      validatorGentxs = mkValidatorGentx {
        home = genesisHome;
        inherit validatorKeys;
        txAccount = "alice";
      };
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

      checks = { };
    };
}
