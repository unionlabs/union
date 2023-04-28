{ ... }: {
  perSystem = { self', pkgs, ... }: {
    checks = { };

    packages =
      let
        src = pkgs.fetchFromGitHub {
          owner = "chainsafe";
          repo = "lodestar";
          rev = "c65e1a428c43f252b99a5fffa77fbc27f224dc07";
          hash = "sha256-Y8Jv6WV066Q0osfNc9+HnwikrZrrtwBzLCz+t3M69bA=";
        };
        nodePackage = pkgs.stdenv.mkDerivation {
          __noChroot = true;
          name = "lodestar-node";
          version = "v1.6.0";
          nativeBuildInputs = with pkgs; [
            python3
            nodejs
            nodePackages.node-gyp-build
            yarn
            (snappy.override { static = true; })
            pkg-config
          ];
          inherit src;
          buildPhase = ''
            export HOME=$(mktemp -d)
            yarn
            export PATH="$(pwd)/node_modules/.bin:$PATH"
            yarn run build
          '';
          installPhase = ''
            mkdir -p $out
            cp -R . $out
          '';
        };
      in
      {
        lodestar-cli = pkgs.writeShellApplication {
          name = "lodestar-cli";
          runtimeInputs = [ pkgs.nodejs ];
          text = ''
            ${nodePackage}/packages/cli/bin/lodestar.js "$@"
          '';
        };
      };

    apps = {
      # @hussein-aitlahcen: arion the following two apps
      lodestar-local = {
        type = "app";
        program = pkgs.writeShellApplication {
          name = "lodestar-local";
          runtimeInputs = [ self'.packages.lodestar-cli pkgs.curl pkgs.jq ];
          text = ''
            ETH_ENDPOINT=http://127.0.0.1:8545
            EXECUTION_ENDPOINT=http://127.0.0.1:8551
            ETH_GENESIS_HASH=$(curl "$ETH_ENDPOINT" \
              -X POST \
              -H 'Content-Type: application/json' \
              -d '{"jsonrpc": "2.0", "id": "1", "method": "eth_getBlockByNumber","params": ["0x0", false]}' | jq -r '.result.hash')
            GENESIS_TIMESTAMP=$(date -d'+10second' +%s)
            lodestar-cli dev \
              --genesisTime "$GENESIS_TIMESTAMP" \
              --genesisEth1Hash "$ETH_GENESIS_HASH" \
              --genesisValidators 8 \
              --startValidators "0..7" \
              --enr.ip6 "127.0.0.1" \
              --eth1.providerUrls "$ETH_ENDPOINT" \
              --execution.urls "$EXECUTION_ENDPOINT" \
              --reset \
              --terminal-total-difficulty-override 0 \
              --params.ALTAIR_FORK_EPOCH 0 \
              --params.BELLATRIX_FORK_EPOCH 0 \
              --params.CAPELLA_FORK_EPOCH 0 \
              --eth1=true \
              --jwt-secret ${./dev-jwt.prv} \
              --rest.namespace="*"
          '';
        };
      };
      geth-local = {
        type = "app";
        program = pkgs.writeShellApplication {
          name = "geth-local";
          runtimeInputs = [ pkgs.go-ethereum ];
          text = ''
            DATADIR=$(mktemp -d)
            ETH_DATADIR=$DATADIR/geth
            cp ${./genesis.json} "$DATADIR/genesis.json"
            geth init --datadir "$ETH_DATADIR" "$DATADIR/genesis.json"
            geth account import --datadir "$ETH_DATADIR" --password /dev/null ${./dev-key0.prv}
            geth account import --datadir "$ETH_DATADIR" --password /dev/null ${./dev-key1.prv}
            geth --vmdebug \
              --datadir "$ETH_DATADIR" \
              --networkid 15 \
              --http \
              --http.api debug,personal,eth,net,web3,txpool,admin,engine,miner --ws --ws.api debug,eth,net,web3,engine \
              --rpc.allow-unprotected-txs \
              --mine \
              --miner.threads=1 \
              --miner.etherbase=0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD \
              --authrpc.addr="127.0.0.1" \
              --http.addr="0.0.0.0" \
              --http.corsdomain '*' \
              --authrpc.jwtsecret ${./dev-jwt.prv} \
              --allow-insecure-unlock \
              --unlock 0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD,0x89b4AB1eF20763630df9743ACF155865600daFF2 \
              --password /dev/null \
              --rpc.gascap 0 \
              --ws.origins "*" \
              --gcmode archive \
              --syncmode=full \
              --maxpeers=0
          '';
        };
      };
    };
  };
}
