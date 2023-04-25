{ ... }: {
  perSystem = { pkgs, self', ... }: {
    packages.genesis =
      let
        uniond = pkgs.lib.getExe self'.packages.uniond;
        chainId = "union-devnet-1";
        N = 3;
      in
      pkgs.runCommand "genesis" { } ''
        				mkdir $out
        				cd $out

        				export HOME=$(pwd)
        				${uniond} init testnet bn254 --chain-id ${chainId} --home .

                for i in {1..${builtins.toString N}}
                do
                  KEYPAIR=`${uniond} genbn`
                  PUBKEY=`echo $KEYPAIR | ${pkgs.jq}/bin/jq ."pub_key"."value"`
                  PUBKEY="{\"@type\":\"/cosmos.crypto.bn254.PubKey\",\"key\":$PUBKEY}"

                  ${uniond} keys add val-$i --keyring-backend test --home . # not saved yet
                  ${uniond} add-genesis-account val-$i 100000000000000000000000000stake --keyring-backend test --home .
                  ${uniond} gentx val-$i 1000000000000000000000stake "bn254" --keyring-backend test --chain-id ${chainId} --home . --ip "0.0.0.0" --pubkey $PUBKEY

                  mv ./config/gentx/gentx*.json ./config/gentx/gen-val-$1.json
                done

                ${uniond} collect-gentxs --home .
                ${uniond} validate-genesis --home .
      '';

    checks = { };
  };
}
