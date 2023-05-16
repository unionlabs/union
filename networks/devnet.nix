{ ... }: {
  perSystem = { devnetConfig, pkgs, system, self', inputs', ... }: {
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
                config = self'.packages.devnet-geth-config;
                genesis = self'.packages.devnet-geth-prysm-genesis;
              };
              prysm-beacon = import ./services/prysm-beacon.nix {
                inherit pkgs;
                prysm-beacon-chain = import ./tools/prysm-beacon-chain.nix { inherit pkgs; inherit system; };
                config = self'.packages.devnet-prysm-config;
                genesis = self'.packages.devnet-geth-prysm-genesis;
              };
              prysm-validator = import ./services/prysm-validator.nix {
                inherit pkgs;
                prysm-validator = import ./tools/prysm-validator.nix { inherit pkgs; inherit system; };
                config = self'.packages.devnet-prysm-config;
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
