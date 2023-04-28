{ ... }: {
  perSystem = { devnetConfig, pkgs, self', inputs', ... }: {
    packages.devnet =
      let
        arion = inputs'.arion.packages.default;
        uniond = pkgs.lib.getExe self'.packages.uniond;

        lodestarService =
          let
            config = pkgs.linkFarm "lodestar-config" [
              { name = "dev-jwt.prv"; path = "${./evm/dev-jwt.prv}"; }
            ];
            script = pkgs.writeShellApplication {
              name = "lodestar-init";
              runtimeInputs = [
                pkgs.curl
                pkgs.jq
                self'.packages.lodestar-cli
                config
              ];
              text = ''
                ETH_ENDPOINT=http://geth:8545
                EXECUTION_ENDPOINT=http://geth:8551
                curl "$ETH_ENDPOINT" \
                                  -X POST \
                                  -H 'Content-Type: application/json' \
                                  -d '{"jsonrpc": "2.0", "id": "1", "method": "eth_getBlockByNumber","params": ["0x0", false]}'
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
                  --rest.address "0.0.0.0" \
                  --rest.cors "*" \
                  --eth1.providerUrls "$ETH_ENDPOINT" \
                  --execution.urls "$EXECUTION_ENDPOINT" \
                  --reset \
                  --terminal-total-difficulty-override 0 \
                  --params.ALTAIR_FORK_EPOCH 0 \
                  --params.BELLATRIX_FORK_EPOCH 0 \
                  --params.CAPELLA_FORK_EPOCH 0 \
                  --eth1=true \
                  --jwt-secret ${config}/dev-jwt.prv \
                  --rest.namespace "*"
              '';
            };
          in
          {
            image = {
              enableRecommendedContents = true;
              contents = [
                pkgs.coreutils
                script
              ];
            };
            service = {
              stop_signal = "SIGINT";
              networks = [ "union-devnet" ];
              ports = [
                # Beacon node rest API
                "9596:9596"
              ];
              command = [
                "sh"
                "-c"
                "${script}/bin/lodestar-init"
              ];
              depends_on = {
                geth = {
                  condition = "service_healthy";
                };
              };
            };
          };

        # @hussein-aitlahcen: maybe move the command part to its own package to run it outside of devnet
        gethService =
          let
            config = pkgs.linkFarm "geth-config" [
              { name = "genesis.json"; path = "${./evm/genesis.json}"; }
              { name = "dev-key0.prv"; path = "${./evm/dev-key0.prv}"; }
              { name = "dev-key1.prv"; path = "${./evm/dev-key1.prv}"; }
              { name = "dev-jwt.prv"; path = "${./evm/dev-jwt.prv}"; }
            ];
            script = pkgs.writeShellApplication {
              name = "geth-init";
              runtimeInputs = [
                pkgs.go-ethereum
                config
              ];
              text = ''
                DATADIR=.
                ETH_DATADIR=$DATADIR/geth
                cp ${config}/genesis.json "$DATADIR/genesis.json"
                geth init --datadir "$ETH_DATADIR" "$DATADIR/genesis.json"
                geth account import --datadir "$ETH_DATADIR" --password /dev/null ${config}/dev-key0.prv
                geth account import --datadir "$ETH_DATADIR" --password /dev/null ${config}/dev-key1.prv
                geth --vmdebug \
                  --datadir "$ETH_DATADIR" \
                  --networkid 15 \
                  --http \
                  --http.api debug,personal,eth,net,web3,txpool,admin,engine,miner --ws --ws.api debug,eth,net,web3,engine \
                  --rpc.allow-unprotected-txs \
                  --mine \
                  --miner.threads 1 \
                  --miner.etherbase 0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD \
                  --authrpc.addr "0.0.0.0" \
                  --http.addr "0.0.0.0" \
                  --http.corsdomain "*" \
                  --http.vhosts "*" \
                  --ws.addr "0.0.0.0" \
                  --ws.origins "*" \
                  --authrpc.vhosts "*" \
                  --authrpc.jwtsecret ${config}/dev-jwt.prv \
                  --allow-insecure-unlock \
                  --unlock 0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD,0x89b4AB1eF20763630df9743ACF155865600daFF2 \
                  --password /dev/null \
                  --rpc.gascap 0 \
                  --gcmode archive \
                  --syncmode full \
                  --maxpeers 0
              '';
            };
          in
          {
            image = {
              enableRecommendedContents = true;
              contents = [
                pkgs.coreutils
                pkgs.curl
                pkgs.jq
                script
              ];
            };
            service = {
              stop_signal = "SIGINT";
              networks = [ "union-devnet" ];
              ports = [
                # Rest HTTP
                "8545:8545"
                # Auth RPC
                "8551:8551"
              ];
              command = [
                "sh"
                "-c"
                "${script}/bin/geth-init"
              ];
              healthcheck = {
                start_period = "5s";
                interval = "10s";
                retries = 4;
                test = [
                  "CMD-SHELL"
                  ''
                    curl http://127.0.0.1:8545 \
                      -X POST \
                      -H 'Content-Type: application/json' \
                      -d '{"jsonrpc": "2.0", "id": "1", "method": "eth_getBlockByNumber","params": ["0x0", false]}' | jq -r '.result.hash' || exit 1
                  ''
                ];
              };
            };
          };

        getNodeID = nodeFile:
          pkgs.runCommand "get-node-id" { } ''
            ${uniond} init testnet bn254 --home .
            cp ${self'.packages.devnet-validator-node-ids}/${nodeFile} ./config/node_key.json
            NODE_ID=$(${uniond} tendermint show-node-id --home .)
            echo -n $NODE_ID > $out
          '';

        mkUniondService = { id }:
          let
            seedNode = builtins.readFile (getNodeID "valnode-0.json");
            # All nodes connects to node 0
            params = if id == 0 then "" else "--p2p.persistent_peers ${seedNode}@uniond-0:26656";
          in
          {
            image = {
              enableRecommendedContents = true;
              contents = [
                pkgs.coreutils
                self'.packages.devnet-genesis
                self'.packages.uniond
                self'.packages.devnet-validator-keys
                self'.packages.devnet-validator-node-ids
              ];
            };
            service = {
              stop_signal = "SIGINT";
              networks = [ "union-devnet" ];
              ports = [
                # CometBLS JSONRPC 26657
                "${toString (26657 + id)}:26657"
                # Cosmos SDK GRPC 9090
                "${toString (9090 + id)}:9090"
                # Cosmos SDK REST 1317
                "${toString (1317 + id)}:1317"
              ];
              command = [
                "sh"
                "-c"
                ''
                  cp -R ${self'.packages.devnet-genesis} .
                  cp ${self'.packages.devnet-validator-keys}/valkey-${toString id}.json ./config/priv_validator_key.json
                  cp ${self'.packages.devnet-validator-node-ids}/valnode-${toString id}.json ./config/node_key.json
                  echo ${params}
                  ${uniond} start --home . ${params} --rpc.laddr tcp://0.0.0.0:26657 --api.address tcp://0.0.0.0:1317 --grpc.address 0.0.0.0:9090
                ''
              ];
            };
          };

        spec = {
          modules = [{
            project.name = "union-devnet";
            networks.union-devnet = { };
            services = (builtins.listToAttrs (builtins.genList
              (i: {
                name = "uniond-${toString i}";
                value = mkUniondService {
                  id = i;
                };
              }
              )
              devnetConfig.validatorCount)) // {
              geth = gethService;
              lodestar = lodestarService;
            };
          }];
        };
        build = arion.build spec;
      in
      pkgs.writeShellApplication {
        NIXBUILDNET_SANDBOX = "relaxed";
        name = "union-devnet";
        runtimeInputs = [ arion ];
        text = ''
          arion --prebuilt-file ${build} up --build --force-recreate -V --always-recreate-deps --remove-orphans
        '';
      };
  };
}
