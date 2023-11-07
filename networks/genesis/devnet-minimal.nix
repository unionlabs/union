#cspell:ignore abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz
{ inputs, ... }: {
  perSystem = { pkgs, inputs', system, get-flake, ... }:
    let
      VALIDATOR_COUNT = 4;
      CHAIN_ID = "union-minimal-1";
      MNEMONIC = "wine parrot nominee girl exchange element pudding grow area twenty next junior come render shadow evidence sentence start rough debate feed all limb real";
      GENESIS_ACCOUNT_NAME = "testkey";

      uniond = pkgs.lib.getExe (get-flake inputs.v0_14_0).packages.${system}.uniond;

      mkNodeId = name:
        pkgs.runCommand "node-id" { } ''
          ${uniond} init testnet bn254 --chain-id ${CHAIN_ID} --home .
          mkdir -p $out
          mv ./config/node_key.json $out/${name}
        '';

      initHome = pkgs.runCommand "genesis-home"
        {
          buildInputs = [ pkgs.moreutils pkgs.dasel ];
        }
        ''
          export HOME=$(pwd)
          mkdir -p $out

          ${uniond} init testnet bn254 --chain-id ${CHAIN_ID} --home $out

          # Add the dev account
          echo ${MNEMONIC} | ${uniond} keys add \
            --recover ${GENESIS_ACCOUNT_NAME} \
            --keyring-backend test \
            --home $out

          dasel put --help

          dasel -f $out/config/genesis.json put -t string -v 12s '.app_state.gov.params.voting_period'
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

      genesisHome = pkgs.lib.foldl
        (home: f: f home)
        initHome
        (
          [
            (mkHome {
              genesisAccounts = builtins.genList (i: "val-${toString i}") VALIDATOR_COUNT;
            })
          ]
        );

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
                ${uniond} gentx val-${toString i} 1000000000000000000000stake "bn254" --keyring-backend test --chain-id ${CHAIN_ID} --home ${home} --ip "0.0.0.0" --pubkey $PUBKEY --moniker validator-${toString i} --output-document $out/valgentx-${
                  toString i
                }.json
              '')
          validatorKeys;

      validatorKeys = mkValidatorKeys { validatorCount = VALIDATOR_COUNT; home = genesisHome; };
      validatorGentxs = mkValidatorGentx {
        home = genesisHome;
        inherit validatorKeys;
      };
      validatorNodeIds = validatorCount: builtins.genList (i: mkNodeId "valnode-${toString i}.json") validatorCount;
    in
    {
      packages.minimal-genesis = pkgs.runCommand "genesis" { } ''
        mkdir $out
        cd $out

        export HOME=$(pwd)
        cp --no-preserve=mode -r ${genesisHome}/* .

        mkdir ./config/gentx
        ${builtins.concatStringsSep "\n" (pkgs.lib.lists.imap0 (i: valGentx: ''
          cp ${valGentx}/valgentx-${toString i}.json ./config/gentx/
        '') validatorGentxs)}

        ${uniond} collect-gentxs --home $HOME 2> /dev/null
        ${uniond} validate-genesis --home $HOME
      '';

      packages.minimal-validator-keys = pkgs.symlinkJoin {
        name = "validator-keys";
        paths = validatorKeys;
      };

      packages.minimal-validator-gentxs = pkgs.symlinkJoin {
        name = "validator-gentxs";
        paths = validatorGentxs;
      };

      packages.minimal-validator-node-ids = pkgs.symlinkJoin {
        name = "validator-node-ids";
        paths = validatorNodeIds VALIDATOR_COUNT;
      };
    };
}
