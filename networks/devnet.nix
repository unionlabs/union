{ inputs, ... }: {
  perSystem = { devnetConfig, pkgs, lib, self', inputs', system, get-flake, mkCi, mkNodeId, dbg, ... }:
    let
      arion = inputs'.arion.packages.default;

      mkCosmosDevnet = import ./mkCosmosDevnet.nix { inherit pkgs dbg; };

      devnet-union = mkCosmosDevnet {
        node = self'.packages.uniond;
        chainId = "union-devnet-1";
        chainName = "union";
        denom = "muno";
        keyType = "bn254";
        validatorCount = 4;
        genesisOverwrites = {
          app_state = {
            staking.params = {
              epoch_length = "8";
              jailed_validator_threshold = "10";
            };
            slashing.params = { signed_blocks_window = "10"; };
          };
        };
        lightClients = [
          self'.packages.ethereum-light-client-minimal
          self'.packages.ethereum-light-client-mainnet
          self'.packages.scroll-light-client
        ];
        cosmwasmContracts = [
          self'.packages.ucs00-pingpong
          self'.packages.ucs01-relay
        ];
        portIncrease = 0;
      };

      devnet-stargaze = mkCosmosDevnet {
        node = self'.packages.starsd;
        chainId = "stargaze-devnet-1";
        chainName = "stargaze";
        denom = "ustars";
        keyType = "ed25519";
        validatorCount = 4;
        lightClients = [
          self'.packages.cometbls-light-client
        ];
        cosmwasmContracts = [
          self'.packages.ucs00-pingpong
          self'.packages.ucs01-relay
        ];
        portIncrease = 100;
      };

      devnet-simd = mkCosmosDevnet {
        node = self'.packages.simd;
        chainId = "simd-devnet-1";
        chainName = "simd";
        denom = "stake";
        keyType = "ed25519";
        validatorCount = 4;
        lightClients = [
          self'.packages.cometbls-light-client
        ];
        cosmwasmContracts = [
          self'.packages.ucs00-pingpong
          self'.packages.ucs01-relay
        ];
        portIncrease = 100;
      };

      devnet-union-minimal = mkCosmosDevnet {
        node = (get-flake inputs.v0_19_0).packages.${system}.uniond;
        chainId = "union-minimal-devnet-1";
        chainName = "union-minimal";
        denom = "muno";
        keyType = "bn254";
        validatorCount = 4;
        portIncrease = 0;
      };

      services = {
        devnet-union = devnet-union.services;

        devnet-simd = devnet-simd.services;

        devnet-stargaze = devnet-stargaze.services;

        devnet-union-minimal = devnet-union-minimal.services;

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

        devnet-union = {
          project.name = "devnet-union";
          services = services.devnet-union;
        };

        devnet-union-minimal = {
          project.name = "devnet-union-minimal";
          services = services.devnet-union-minimal;
        };

        devnet-simd = {
          project.name = "devnet-simd";
          services = services.devnet-simd;
        };

        devnet-stargaze = {
          project.name = "devnet-stargaze";
          services = services.devnet-stargaze;
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

        devnet-stargaze = {
          modules = [ (modules.devnet-stargaze // { networks.devnet-stargaze = { }; }) ];
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

        devnet-stargaze = arion.build specs.devnet-stargaze;

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
        devnet-union-home = mkCi false (devnet-union.devnet-home);

        devnet-simd = mkCi (system == "x86_64-linux") (mkArionBuild build.devnet-simd "devnet-simd");
        devnet-simd-home = mkCi false (devnet-simd.devnet-home);

        devnet-stargaze = mkCi (system == "x86_64-linux") (mkArionBuild build.devnet-stargaze "devnet-stargaze");
        devnet-stargaze-home = mkCi false (devnet-stargaze.devnet-home);

        devnet-eth = mkCi (system == "x86_64-linux") (mkArionBuild build.devnet-eth "devnet-eth");

        voyager-queue = mkCi false (mkArionBuild build.voyager-queue "voyager-queue");

        # FIXME: This shouldn't be defined in this file
        devnet-eth-config = pkgs.linkFarm "devnet-eth-config" [
          { name = "genesis.json"; path = "${./genesis/devnet-eth/genesis.json}"; }
          { name = "dev-key0.prv"; path = "${./genesis/devnet-eth/dev-key0.prv}"; }
          { name = "dev-key1.prv"; path = "${./genesis/devnet-eth/dev-key1.prv}"; }
          { name = "dev-jwt.prv"; path = "${./genesis/devnet-eth/dev-jwt.prv}"; }
        ];
      };

      _module.args.networks.modules = modules;
    };
}
