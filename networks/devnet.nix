{ ... }: {
  perSystem = { devnetConfig, pkgs, self', inputs', ... }: {
    packages.devnet =
      let
        arion = inputs'.arion.packages.default;
        uniond = pkgs.lib.getExe self'.packages.uniond;
        lodestarService =
          {
            image = {
              enableRecommendedContents = true;
              contents = [
                pkgs.coreutils
                self'.packages.devnet-lodestar-init
              ];
            };
            service = {
              stop_signal = "SIGINT";
              networks = [ "union-devnet" ];
              ports = [
                # Beacon node rest API
                "9596:9596"
              ];
              command = [ "sh" "-c" "${self'.packages.devnet-geth-init}/bin/lodestar-init" ];
              depends_on = {
                geth = {
                  condition = "service_healthy";
                };
              };
            };
          };

        gethService =
          {
            image = {
              enableRecommendedContents = true;
              contents = [
                pkgs.coreutils
                pkgs.curl
                pkgs.jq
                self'.packages.devnet-geth-init
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
              command = [ "sh" "-c" "${self'.packages.devnet-geth-init}/bin/geth-init" ];
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
            # All nodes connect to node 0
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
        name = "union-devnet";
        runtimeInputs = [ arion ];
        text = ''
          arion --prebuilt-file ${build} up --build --force-recreate -V --always-recreate-deps --remove-orphans
        '';
      };
  };
}
