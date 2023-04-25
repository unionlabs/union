{ ... }: {
  perSystem = { pkgs, self', ... }: {
    packages.genesis =
      let
        chainId = "union-devnet-1";
        uniond = self'.packages.uniond;
        N = 3;
      in
      pkgs.runCommand "genesis" { } ''
        				mkdir $out
        				cd $out

        				export HOME=$(pwd)
        				${uniond}/bin/uniond init testnet bn254 --chain-id ${chainId} --home .

                for i in {1..${builtins.toString N}}
                do
                  KEYPAIR=`${uniond}/bin/uniond genbn`
                  PUBKEY=`echo $KEYPAIR | ${pkgs.jq}/bin/jq ."pub_key"."value"`
                  PUBKEY="{\"@type\":\"/cosmos.crypto.bn254.PubKey\",\"key\":$PUBKEY}"

                  ${uniond}/bin/uniond keys add val-$i --keyring-backend test --home . # not saved yet
                  ${uniond}/bin/uniond add-genesis-account val-$i 100000000000000000000000000stake --keyring-backend test --home .
                  ${uniond}/bin/uniond gentx val-$i 1000000000000000000000stake "bn254" --keyring-backend test --chain-id ${chainId} --home . --ip "0.0.0.0" --pubkey $PUBKEY

                  mv ./config/gentx/gentx*.json ./config/gentx/gen-val-$1.json
                done

                ${uniond}/bin/uniond collect-gentxs --home .
                ${uniond}/bin/uniond validate-genesis --home .
      '';

    checks = { };
  };
}
