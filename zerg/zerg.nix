{ ... }: {
  perSystem = { self', pkgs, system, config, crane, stdenv, ... }:
    let
      uniond = pkgs.lib.getExe self'.packages.uniond;

      zerg = crane.buildWorkspaceMember {
        crateDirFromRoot = "zerg";
        additionalTestSrcFilter = path: _:
          pkgs.lib.hasPrefix "hubble/src/graphql" path;
        cargoTestExtraAttrs = {
          partitions = 1;
          partitionType = "count";
        };
      };

    in
    {
      packages = {
        zerg = zerg.packages.zerg;

        generate-testnet-accounts = pkgs.writeShellApplication {
          name = "generate-testnet-accounts";
          runtimeInputs = [ ];
          text = ''
            usage() {
              printf "\
              Usage: generate-funded-testnet-accounts [OPTION]... \n\
              \n\
              Options: \n\
                -n, --n-accounts Number of accounts to generate and fund \n\
                -a, --amount     Amount of funds to send (Default: 1000) \n\
                -d, --denom      Denom (Default: muno) \n\
                -h, --home       Home directory of the chain \n\
                --node           Rpc endpoint of the node (Default: \"http://localhost:26657\") \n\
                -k, --key-name    Key to use when sending the funds \n\
                -c, --chain-id   Chain id to use \n\
                --no-fund        Don't fund the accounts \n\
                --help           Print help \n\
              "
              exit "$1"
            }

            AMOUNT=1000
            DENOM=muno
            NUM_OF_ACCOUNTS=""
            NO_FUND=""
            HOME=""
            NODE="http://localhost:26657"
            KEY_NAME=""
            CHAIN_ID=""

            while [[ $# -gt 0 ]]; do
              case $1 in
                -n|--n-accounts)
                  NUM_OF_ACCOUNTS="$2"
                  shift
                  shift
                  ;;
                -a|--amount)
                  AMOUNT="$2"
                  shift
                  shift
                  ;;
                -d|--denom)
                  DENOM="$2"
                  shift
                  shift
                  ;;
                --no-fund)
                  NO_FUND=1
                  shift
                  ;;
                -h|--home)
                  HOME="$2"
                  shift
                  shift
                  ;;
                -c|--chain-id)
                  CHAIN_ID="$2"
                  shift
                  shift
                  ;;
                --node)
                  NODE="$2"
                  shift
                  shift
                  ;;
                -k|--key-name)
                  KEY_NAME="$2"
                  shift
                  shift
                  ;;
                --help)
                  usage 0
                  ;;
                esac
              done

              if [[ -z "$NUM_OF_ACCOUNTS" ]]; then
                echo "--n-accounts must be specified."
                usage 1
              fi

              if [[ -z "$KEY_NAME" ]] && [[ -z "$NO_FUND" ]]; then
                echo "--key-name must be specified since funds are gonna be sent to accounts."
                echo "Use --no-fund to disable funding the accounts."
                usage 1
              fi

              if [[ -z "$CHAIN_ID" ]] && [[ -z "$NO_FUND" ]]; then
                echo "--chain-id must be specified since funds are gonna be sent to accounts."
                usage 1
              fi

              if [[ -z "$HOME" ]] && [[ -z "$NO_FUND" ]]; then
                echo "--home must be specified since funds are gonna be sent to accounts."
                usage 1
              fi

              ACCOUNTS=""
              for i in $(seq "$NUM_OF_ACCOUNTS"); do
                ACC_NAME="zerg-gen-acc-$i"
                ADDR=$(echo y | ${uniond} keys add "$ACC_NAME" --keyring-backend test --home "$HOME" --output json | jq .address -r)
                ACCOUNTS="$ACCOUNTS $ADDR"
                echo "---------------------"
                echo "+ Generated $ADDR"
                echo "+ Private key"
                echo y | ${uniond} keys export "$ACC_NAME" --unsafe --unarmored-hex --keyring-backend test
              done

              if [[ -z "$NO_FUND" ]]; then
                echo ".. Sending $AMOUNT$DENOM to $ACCOUNTS by using key $KEY_NAME."
                eval ${uniond} tx bank multi-send "$KEY_NAME" "$ACCOUNTS" "$AMOUNT$DENOM" --home "$HOME" --node "$NODE" --keyring-backend test -y --chain-id "$CHAIN_ID" --gas auto --gas-adjustment=1.3
              fi
          '';
        };
      };
    };
}
