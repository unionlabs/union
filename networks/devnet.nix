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

      wasmd-services = (builtins.listToAttrs (builtins.genList
        (id: {
          name = "wasmd-${toString id}";
          value = import ./services/wasmd.nix {
            inherit pkgs;
            inherit id;
            wasmd = self'.packages.wasmd;
            devnet-genesis = self'.packages.wasmd-genesis;
            devnet-validator-keys = self'.packages.wasmd-validator-keys;
            devnet-validator-node-ids = self'.packages.wasmd-validator-node-ids;
          };
        })
        devnetConfig.validatorCount));

      sepolia-services = {
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
        project.name = "devnet";
        services = uniond-services // sepolia-services;
      };

      union = {
        project.name = "union";
        services = uniond-services;
      };

      sepolia = {
        project.name = "sepolia";
        services = sepolia-services;
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

      spec-wasmd = {
        modules = [{
          project.name = "wasmd-devnet";
          networks.wasmd-devnet = { };
          services = wasmd-services;
        }];
      };

      spec-evm = {
        modules = [{
          project.name = "union-devnet-evm";
          networks.union-devnet = { };
          services = sepolia-services;
        }];
      };

      build = arion.build spec;

      build-evm = arion.build spec-evm;

      build-cosmos = arion.build spec-cosmos;

      build-wasmd = arion.build spec-wasmd;
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

      packages.devnet-wasmd =
        pkgs.writeShellApplication {
          name = "wasmd-devnet";
          runtimeInputs = [ arion ];
          text = ''
            arion --prebuilt-file ${build-wasmd} up --build --force-recreate -V --always-recreate-deps --remove-orphans
          '';
        };

      _module.args.networks = {
        inherit devnet union sepolia;
      };
    };
}
