{ ... }: {
  perSystem = { devnetConfig, pkgs, self', inputs', ... }:
    let
      arion = inputs'.arion.packages.default;

      uniond = pkgs.lib.getExe self'.packages.uniond;

      uniond-services = (builtins.listToAttrs (builtins.genList
        (id: {
          name = "uniond-${toString id}";
          value = import ./services/uniond.nix {
            inherit pkgs;
            inherit id;
            uniond = self'.packages.uniond;
            devnet-genesis = self'.packages.devnet-genesis;
            devnet-validator-keys = self'.packages.devnet-validator-keys;
            devnet-validator-node-ids = self'.packages.devnet-validator-node-ids;
          };
        })
        devnetConfig.validatorCount));

      evm-services = {
        geth = import ./services/geth.nix {
          inherit pkgs;
          config = self'.packages.devnet-evm-config;
        };
        lodestar = import ./services/lodestar.nix {
          inherit pkgs;
          config = self'.packages.devnet-evm-config;
          validatorCount = devnetConfig.ethereum.beacon.validatorCount;
        };
      };

      devnet = {
        project.name = "union-devnet";
        services = uniond-services // evm-services;
      };

      spec = {
        modules = [ (devnet // { networks.union-devnet = { }; }) ];
      };

      spec-cosmos = {
        modules = [{
          project.name = "union-devnet-cosmos";
          networks.union-devnet = { };
          services = uniond-services;
        }];
      };

      spec-evm = {
        modules = [{
          project.name = "union-devnet-evm";
          networks.union-devnet = { };
          services = evm-services;
        }];
      };

      build = arion.build spec;

      build-evm = arion.build spec-evm;

      build-cosmos = arion.build spec-cosmos;

      instantiate-cw20-ics20 =
        pkgs.writeShellApplication {
          name = "instantiate-cw20-ics20";
          runtimeInputs = [];
          text =
        ''
          while ! ${uniond} tx wasm instantiate2 1 \
            '{
              "default_timeout":300,
              "gov_contract":"union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2",
              "allowlist":[],
              "channel":{
                "endpoint":{
                  "port_id": "",
                  "channel_id":"channel-0"
                },
                "counterparty_endpoint":{
                  "port_id":"transfer",
                  "channel_id":"channel-0"
                },
                "order":"ORDER_UNORDERED",
                "version":"ics20-1",
                "connection_id":"connection-0"
              }
            }' \
            61616161 \
            --label cw20-ics20-test \
            --gas=auto \
            --gas-adjustment=1.3 -y  \
            --admin union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2 \
            --keyring-backend test \
            --from testkey \
            --chain-id union-devnet-1 \
            --home ${self'.packages.devnet-genesis}
          do
            sleep 1
          done
        '';
        };

      deploy-contracts =
        pkgs.writeShellApplication {
          name = "deploy-contracts";
          runtimeInputs = [];
          text = ''
            while ! ${self'.packages.evm-devnet-deploy}/bin/evm-devnet-deploy
            do
              sleep 1
            done 
          '';
        };
    in
    {
      packages.devnet =
        pkgs.writeShellApplication {
          name = "union-devnet";
          runtimeInputs = [ arion ];
          text = ''
            arion --prebuilt-file ${build} up --build --force-recreate -V --always-recreate-deps --remove-orphans
          '';
        };

      packages.devnet-evm =
        pkgs.writeShellApplication {
          name = "union-devnet-evm";
          runtimeInputs = [ arion ];
          text = ''
            arion --prebuilt-file ${build-evm} up --build --force-recreate -V --always-recreate-deps --remove-orphans
          '';
        };

      packages.devnet-cosmos =
        pkgs.writeShellApplication {
          name = "union-devnet-cosmos";
          runtimeInputs = [ arion ];
          text = ''
            arion --prebuilt-file ${build-cosmos} up --build --force-recreate -V --always-recreate-deps --remove-orphans
          '';
        };

      packages.devnet-demo =
        pkgs.writeShellApplication {
          name = "union-devnet-demo";
          runtimeInputs = [ ];
          text = ''
            (trap 'kill 0' SIGINT; \
              ${self'.packages.devnet}/bin/union-devnet & \
              ${instantiate-cw20-ics20}/bin/instantiate-cw20-ics20 & \
              ${deploy-contracts}/bin/deploy-contracts & \
              wait)
          '';
        };

      _module.args.networks = {
        inherit devnet;
      };
    };
}
