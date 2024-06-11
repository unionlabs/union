{ ... }: {
  perSystem = { self', pkgs, system, config, crane, stdenv, ... }:
    let
      uniond = pkgs.lib.getExe self'.packages.uniond;

      zerg = crane.buildWorkspaceMember {
        crateDirFromRoot = "zerg";
        cargoTestExtraAttrs = {
          partitions = 1;
          partitionType = "count";
        };
        extraEnv = {
          doNotLinkInheritedArtifacts = true;
        };
      };

    in
    {
      packages = {
        zerg = zerg.packages.zerg;

        generate-testnet-accounts = pkgs.writeShellApplicationWithArgs {
          name = "generate-testnet-accounts";
          runtimeInputs = [ pkgs.jq uniond ];
          arguments = [
            {
              arg = "n_accounts";
              help = "Number of accounts to generate and fund";
              required = true;
            }
            {
              arg = "amount";
              help = "Amount of funds to send";
              default = "1000";
            }
            {
              arg = "denom";
              default = "muno";
            }
            {
              arg = "key_name";
              help = "Key to use when sending the funds";
            }
            {
              arg = "no_fund";
              help = "Don't fund the accounts";
              type = "flag";
            }
            {
              arg = "chain_id";
            }
            {
              arg = "home";
              help = "Chain home";
            }
            {
              arg = "node";
              help = "Union node's rpc endpoint";
              default = "http://localhost:26657";
            }
          ];
          text = ''
            if [[ -z "$argc_key_name" ]] && [[ -z "$argc_no_fund" ]]; then
              echo "--key-name must be specified since funds are gonna be sent to accounts."
              echo "Use --no-fund to disable funding the accounts."
              usage 1
            fi

            if [[ -z "$argc_chain_id" ]] && [[ -z "$argc_no_fund" ]]; then
              echo "--chain-id must be specified since funds are gonna be sent to accounts."
              usage 1
            fi

            if [[ -z "$argc_home" ]] && [[ -z "$argc_no_fund" ]]; then
              echo "--home must be specified since funds are gonna be sent to accounts."
              usage 1
            fi

            accounts=""
            for i in $(seq "$argc_n_accounts"); do
              acc_name="zerg-gen-acc-$i"
              addr=$(echo y | ${uniond} keys add "$acc_name" --keyring-backend test --home "$argc_home" --output json | jq .address -r)
              accounts="$accounts $addr"
              echo "---------------------"
              echo "+ Generated $addr"
              echo "+ Private key"
              echo y | ${uniond} keys export "$acc_name" --unsafe --unarmored-hex --keyring-backend test --home "$argc_home"
            done

            if [[ -z "$argc_no_fund" ]]; then
              echo ".. Sending $argc_amount$argc_denom to $accounts by using key $argc_key_name."
              eval ${uniond} tx bank multi-send "$argc_key_name" "$accounts" "$argc_amount$argc_denom" --home "$argc_home" --node "$argc_node" --keyring-backend test -y --chain-id "$argc_chain_id" --gas auto --gas-adjustment=1.3
            fi
          '';
        };
      };
    };
}
