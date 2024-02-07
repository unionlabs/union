{ inputs, ... }: {
  perSystem = { devnetConfig, pkgs, lib, self', inputs', system, get-flake, mkCi, ... }:
    let
      arion = inputs'.arion.packages.default;

      services = {
        devnet-union = (builtins.listToAttrs (builtins.genList
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

        devnet-simd = (builtins.listToAttrs (builtins.genList
          (id: {
            name = "simd-${toString id}";
            value = import ./services/simd.nix {
              inherit pkgs;
              inherit id;
              simd = self'.packages.simd;
              simd-genesis = self'.packages.simd-genesis;
              simd-validator-keys = self'.packages.simd-validator-keys;
              simd-validator-node-ids = self'.packages.simd-validator-node-ids;
            };
          })
          devnetConfig.validatorCount));

        union-testnet-genesis = (builtins.listToAttrs (builtins.genList
          (id: {
            name = "uniond-${toString id}";
            value = import ./services/unionvisor.nix {
              inherit pkgs;
              inherit id;
              uniond = (get-flake inputs.v0_15_0).packages.${system}.uniond;
              unionvisor = self'.packages.unionvisor;
              devnet-genesis = self'.packages.minimal-genesis;
              devnet-validator-keys = self'.packages.minimal-validator-keys;
              devnet-validator-node-ids = self'.packages.minimal-validator-node-ids;
              network = "union-minimal-1";
              bundle = self'.packages.bundle-testnet-next;
            };
          })
          4));

        devnet-eth = {
          geth = import ./services/geth.nix {
            inherit pkgs;
            config = self'.packages.devnet-eth-config;
          };
          lodestar = import ./services/lodestar.nix {
            inherit pkgs;
            config = self'.packages.devnet-eth-config;
            validatorCount = devnetConfig.ethereum.beacon.validatorCount;
          };
        };

        postgres = {
          postgres = import ./services/postgres.nix { inherit lib pkgs; };
        };

        # hasura = import ./services/hasura.nix {
        #   inherit lib pkgs;
        # };
        # hubble = { hubble = import ./services/hubble.nix { inherit lib; image = self'.packages.hubble-image; }; };
      };

      modules = {
        full-dev-setup = {
          project.name = "full-dev-setup";
          services = services.devnet-eth // services.devnet-union // services.postgres;
        };

        devnet-minimal = {
          project.name = "devnet-minimal";
          services = services.uniond-testnet-genesis;
        };

        devnet-union = {
          project.name = "devnet-union";
          services = services.devnet-union;
        };

        devnet-simd = {
          project.name = "devnet-simd";
          services = services.devnet-simd;
        };

        devnet-eth = {
          project.name = "devnet-eth";
          services = services.devnet-eth;
        };

        postgres = {
          project.name = "postgres";
          services = services.postgres;
        };
      };

      specs = {
        full-dev-setup = {
          modules = [ (modules.full-dev-setup // { networks.full-dev-setup = { }; }) ];
        };

        devnet-union = {
          modules = [ (modules.devnet-union // { networks.devnet-union = { }; }) ];
        };

        devnet-simd = {
          modules = [ (modules.devnet-simd // { networks.devnet-simd = { }; }) ];
        };

        devnet-eth = {
          modules = [ (modules.devnet-eth // { networks.devnet-eth = { }; }) ];
        };

        voyager-queue = {
          modules = [ modules.postgres ];
        };
      };

      build = {
        full-dev-setup = arion.build specs.full-dev-setup;

        devnet-union = arion.build specs.devnet-union;

        devnet-simd = arion.build specs.devnet-simd;

        devnet-eth = arion.build specs.devnet-eth;

        voyager-queue = arion.build specs.voyager-queue;
      };

      mkArionBuild = target: name: pkgs.writeShellApplication {
        inherit name;
        runtimeInputs = [ arion ];
        text = ''
          arion --prebuilt-file ${target} up --build --force-recreate -V --always-recreate-deps --remove-orphans
        '';
      };
    in
    {
      packages = {
        full-dev-setup = mkCi (system == "x86_64-linux") (mkArionBuild build.full-dev-setup "full-dev-setup");
        devnet-union = mkCi (system == "x86_64-linux") (mkArionBuild build.devnet-union "devnet-union");
        devnet-simd = mkCi (system == "x86_64-linux") (mkArionBuild build.devnet-simd "devnet-simd");
        devnet-eth = mkCi (system == "x86_64-linux") (mkArionBuild build.devnet-eth "devnet-eth");
        voyager-queue = mkCi false (mkArionBuild build.voyager-queue "voyager-queue");
      };

      _module.args.networks.modules = modules;
    };
}
