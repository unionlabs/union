{ ... }: {
  perSystem = { devnetConfig, pkgs, self', inputs', ... }:
    let
      arion = inputs'.arion.packages.default;

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
          config = self'.packages.devnet-geth-config;
          genesis = self'.packages.devnet-geth-prysm-genesis;
        };
        prysm-beacon = import ./services/prysm-beacon.nix {
          inherit pkgs;
          prysm-beacon-chain = self'.packages.prysm-beacon-chain;
          config = self'.packages.devnet-prysm-config;
          genesis = self'.packages.devnet-geth-prysm-genesis;
        };
        prysm-validator = import ./services/prysm-validator.nix {
          inherit pkgs;
          prysm-validator = self'.packages.prysm-validator;
          config = self'.packages.devnet-prysm-config;
        };
      };

      spec = {
        modules = [{
          project.name = "union-devnet";
          networks.union-devnet = { };
          services = uniond-services // evm-services;
        }];
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
    };
}
