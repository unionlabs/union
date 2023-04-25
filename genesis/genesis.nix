{ ... }: {
  perSystem = { pkgs, self', ... }: {
    packages.genesis =
      let
        chainId = "union-devnet-1";
        uniond = self'.packages.uniond;
        N = 10;
      in
      pkgs.runCommand "genesis" { } ''
        				mkdir $out
        				cd $out
        				export HOME=$(pwd)
                
        				${uniond}/bin/uniond init testnet bn254 --chain-id ${chainId} --home ./master
                mkdir val-con-keypairs

                for i in {1..${builtins.toString N}}
                do
                  KEYPAIR=`${uniond}/bin/uniond genbn`
                  PUBKEY=`echo $KEYPAIR | ${pkgs.jq}/bin/jq ."pub_key"."value"`

                  PUBKEY="{\"@type\":\"/cosmos.crypto.bn254.PubKey\",\"key\":$PUBKEY}"
                  ${uniond}/bin/uniond keys add val-$i --keyring-backend test --home ./master # not saved yet
                  ${uniond}/bin/uniond add-genesis-account val-$i 100000000000000000000000000stake --keyring-backend test --home ./master
                  echo "hello"
                  echo $PUBKEY
                  ${uniond}/bin/uniond gentx val-$i 1000000000000000000000stake "bn254" --keyring-backend test --chain-id ${chainId} --home ./master --ip "0.0.0.0" --pubkey $PUBKEY

                  mv ./master/config/gentx/gentx*.json ./master/config/gentx/gen-val-$1.json

                done

                ${uniond}/bin/uniond collect-gentxs --home ./master
                ${uniond}/bin/uniond validate-genesis --home ./master
                  
                # for validators

            #     mkdir master
        				# ${uniond}/bin/uniond init testnet bn254 --chain-id ${chainId} --home ./master
                
            #     for i in {1..${builtins.toString N}}
            #     do
            #       export HOME=$(pwd)
          		# 		mkdir val-$i
          		# 		${uniond}/bin/uniond init testnet bn254 --chain-id ${chainId} --home ./val-$i
            #       ${uniond}/bin/uniond keys add val-$i --keyring-backend test --home ./val-$i # not saved yet


            #       ${uniond}/bin/uniond add-genesis-account val-$i 100000000000000000000000000stake --keyring-backend test --home ./val-$i
            #       ${uniond}/bin/uniond keys list --home ./val-$i 
            #       # PUBKEY=`${uniond}/bin/uniond keys show val-$i --address --home ./val-$i` 
            #       # ${uniond}/bin/uniond add-genesis-account $PUBKEY 100000000000000000000000000stake --keyring-backend test --home ./master
            #       # ${uniond}/bin/uniond gentx val-$i 1000000000000000000000stake "bn254" --keyring-backend test --chain-id ${chainId} --home ./val-$i --ip "0.0.0.0"
            #       # cp ./val-$i/gentx/* ./master/gentx
            #     done
            #     # end

      '';

    checks = { };
  };
}
