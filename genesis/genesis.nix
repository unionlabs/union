{ ... }: {
  perSystem = { pkgs, self', ... }: {
    packages.genesis =
      let
        chainId = "union-devnet-1";
        uniond = self'.packages.uniond;
        KEY = "val-1";
        N = 10;
      in
      pkgs.runCommand "genesis" { } ''
        				mkdir $out
        				cd $out
        				export HOME=$(pwd)
                # for validators
                for i in {1..${builtins.toString N}}
                do
          				mkdir val-$i
          				${uniond}/bin/uniond init testnet bn254 --chain-id ${chainId} --home ./val-$i
                  ${uniond}/bin/uniond keys add ${KEY} --keyring-backend test --home ./val-$i # not saved yet
                  ${uniond}/bin/uniond add-genesis-account ${KEY} 100000000000000000000000000stake --keyring-backend test --home ./val-$i
                  ${uniond}/bin/uniond gentx ${KEY} 1000000000000000000000stake "bn254" --keyring-backend test --chain-id ${chainId} --home ./val-$i --ip "0.0.0.0"
                done
                # end

      '';

    checks = { };
  };
}
