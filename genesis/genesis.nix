{ ... }: {
  perSystem = { pkgs, self', ... }: {
    packages = {

      genesis =
        let
          chainId = "union-devnet-1";
          uniond = self'.packages.uniond;
          KEY = "val-1";
        in
        pkgs.runCommand "genesis" { } ''
          				mkdir $out
          				cd $out
          				export HOME=$(pwd)
          				

                  # for validators
          				mkdir val-1
          				${uniond}/bin/uniond init testnet bn254 --chain-id ${chainId} --home ./val-1
                  ${uniond}/bin/uniond keys add ${KEY} --keyring-backend test --home ./val-1 # not saved yet
                  # end

                  ${uniond}/bin/uniond add-genesis-account ${KEY} 100000000000000000000000000stake --keyring-backend test --home ./val-1
                  ${uniond}/bin/uniond gentx ${KEY} 1000000000000000000000stake "bn254" --keyring-backend test --chain-id ${chainId} --home ./val-1 --ip "0.0.0.0"
        '';
    };

    checks = { };
  };
}
