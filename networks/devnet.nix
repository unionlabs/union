{ ... }: {
  perSystem = { devnetConfig, pkgs, self', inputs', ... }: {
    packages.devnet =
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

        spec = {
          modules = [{
            project.name = "union-devnet";
            networks.union-devnet = { };
            services = uniond-services // {
              geth = import ./services/geth.nix {
                inherit pkgs;
                geth-init = self'.packages.devnet-geth-init;
              };
              lodestar = import ./services/lodestar.nix {
                inherit pkgs;
                lodestar-cli = self'.packages.lodestar-cli;
                config = self'.packages.devnet-lodestar-config;
              };
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
