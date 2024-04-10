{ inputs, ... }: {
  perSystem = { devnetConfig, pkgs, lib, self', inputs', system, get-flake, mkCi, mkNodeId, dbg, ... }:
    let
      arion = inputs'.arion.packages.default;

      mkCosmosDevnet = import ./mkCosmosDevnet.nix { inherit pkgs dbg; };

      cosmwasmContracts = [
        self'.packages.ucs00-pingpong
        self'.packages.ucs01-relay
        self'.packages.ucs02-nft
        self'.packages.cw721-base
      ];

      devnet-union = dbg (mkCosmosDevnet {
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
            tokenfactory.params = {
              denom_creation_fee = [
                {
                  denom = "muno";
                  amount = "10000000";
                }
              ];
            };
          };
        };
        lightClients = [
          self'.packages.ethereum-light-client-minimal
          self'.packages.ethereum-light-client-mainnet
          self'.packages.scroll-light-client
        ];
        inherit cosmwasmContracts;
        portIncrease = 0;
      });

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
        inherit cosmwasmContracts;
        portIncrease = 100;
      };

      devnet-osmosis = mkCosmosDevnet {
        node = self'.packages.osmosisd;
        chainId = "osmosis-devnet-1";
        chainName = "osmosis";
        denom = "uosmo";
        keyType = "ed25519";
        validatorCount = 4;
        lightClients = [
          self'.packages.cometbls-light-client
        ];
        cosmwasmContracts = [
          self'.packages.ucs00-pingpong
          self'.packages.ucs01-relay
        ];
        portIncrease = 200;
        sdkVersion = 47;
        genesisOverwrites = {
          app_state = {
            tokenfactory.params = {
              denom_creation_fee = [
                {
                  denom = "uosmo";
                  amount = "10000000";
                }
              ];
            };
          };
        };
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
        inherit cosmwasmContracts;
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
        genesisOverwrites = {
          app_state = {
            gov.params = {
              max_deposit_period = "12s";
              voting_period = "18s";
              expedited_voting_period = "6s";
            };
            tokenfactory.params = {
              denom_creation_fee = [
                {
                  denom = "muno";
                  amount = "10000000";
                }
              ];
            };
          };
        };
        extraPackages = [ self'.packages.unionvisor self'.packages.bundle-testnet-next ];
        startCommandOverwrite =
          ''
            mkdir .unionvisor

            export UNIONVISOR_ROOT=$(pwd)/.unionvisor
            export UNIONVISOR_BUNDLE=${self'.packages.bundle-testnet-next}

            ${pkgs.lib.getExe self'.packages.unionvisor} init \
              --moniker union-devnet-minimal \
              --network union-minimal-devnet-1 \
              --seeds "" \

            cp --no-preserve=mode -RL home/* .unionvisor/home

            ${pkgs.lib.getExe self'.packages.unionvisor} run \
              --poll-interval 1000 \
              -- \
              $$params \
              --rpc.laddr tcp://0.0.0.0:26657 \
              --api.enable true \
              --rpc.unsafe \
              --api.address tcp://0.0.0.0:1317 \
              --grpc.address 0.0.0.0:9090
          '';
      };

      services = {
        devnet-union = devnet-union.services;
        devnet-simd = devnet-simd.services;
        devnet-stargaze = devnet-stargaze.services;
        devnet-osmosis = devnet-osmosis.services;

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

      mkNamedModule = name: {
        ${name} = {
          project.name = name;
          services = services.${name};
        };
      };

      modules = {
        full-dev-setup = {
          project.name = "full-dev-setup";
          services = services.devnet-eth // services.devnet-union // services.postgres;
        };
      }
      // mkNamedModule "postgres"
      // mkNamedModule "devnet-eth"
      // mkNamedModule "devnet-stargaze"
      // mkNamedModule "devnet-osmosis"
      // mkNamedModule "devnet-simd"
      // mkNamedModule "devnet-union-minimal"
      // mkNamedModule "devnet-union";

      mkNamedSpec = name: {
        ${name} = {
          modules = [ modules.${name} ];
        };
      };

      specs = {
        voyager-queue = {
          modules = [ modules.postgres ];
        };
      }
      // mkNamedSpec "full-dev-setup"
      // mkNamedSpec "devnet-eth"
      // mkNamedSpec "devnet-stargaze"
      // mkNamedSpec "devnet-osmosis"
      // mkNamedSpec "devnet-simd"
      // mkNamedSpec "devnet-union-minimal"
      // mkNamedSpec "devnet-union";

      mkNamedBuild = name: {
        ${name} = arion.build specs.${name};
      };

      build = mkNamedBuild "full-dev-setup"
        // mkNamedBuild "voyager-queue"
        // mkNamedBuild "devnet-eth"
        // mkNamedBuild "devnet-stargaze"
        // mkNamedBuild "devnet-osmosis"
        // mkNamedBuild "devnet-simd"
        // mkNamedBuild "devnet-union-minimal"
        // mkNamedBuild "devnet-union";

      mkArionBuild = name: ciCondition: {
        ${name} = mkCi ciCondition (pkgs.writeShellApplication {
          inherit name;
          runtimeInputs = [ arion ];
          text = ''
            arion --prebuilt-file ${build.${name}} up --build --force-recreate -V --always-recreate-deps --remove-orphans
          '';
        });
      };
    in
    {
      packages = {
        devnet-union-home = mkCi false (devnet-union.devnet-home);
        devnet-simd-home = mkCi false (devnet-simd.devnet-home);
        devnet-stargaze-home = mkCi false (devnet-stargaze.devnet-home);
        devnet-osmosis-home = mkCi false (devnet-osmosis.devnet-home);

        # FIXME: This shouldn't be defined in this file
        devnet-eth-config = pkgs.linkFarm "devnet-eth-config" [

          { name = "genesis.json"; path = "${./genesis/devnet-eth/genesis.json}"; }
          { name = "dev-key0.prv"; path = "${./genesis/devnet-eth/dev-key0.prv}"; }
          { name = "dev-key1.prv"; path = "${./genesis/devnet-eth/dev-key1.prv}"; }
          { name = "dev-jwt.prv"; path = "${./genesis/devnet-eth/dev-jwt.prv}"; }
        ];
      }
      // (mkArionBuild "full-dev-setup" (system == "x86_64-linux"))
      // (mkArionBuild "voyager-queue" false)
      // (mkArionBuild "devnet-union" (system == "x86_64-linux"))
      // (mkArionBuild "devnet-simd" (system == "x86_64-linux"))
      // (mkArionBuild "devnet-stargaze" (system == "x86_64-linux"))
      // (mkArionBuild "devnet-osmosis" (system == "x86_64-linux"))
      // (mkArionBuild "devnet-eth" (system == "x86_64-linux"))
      // (mkArionBuild "devnet-union-minimal" (system == "x86_64-linux"));

      _module.args.networks.modules = modules;
    };
}
