{ ... }: {
  perSystem = { pkgs, self', ... }: {
    packages = {

      genesis =
        let
          chainId = "union-devnet-1";
          uniond = self'.packages.uniond;
        in
        pkgs.runCommand "genesis" { } ''
          				mkdir $out
          				cd $out
          				export HOME=$(pwd)
          				
          				mkdir val-1
          				${uniond}/bin/uniond init testnet bn254 --chain-id ${chainId} --home ./val-1
            			'';
    };

    checks = { };
  };
}
