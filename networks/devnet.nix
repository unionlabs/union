{ inputs, ... }: {
  perSystem = { devnetConfig, pkgs, lib, self', nix-filter, inputs', system, get-flake, mkCi, mkNodeId, dbg, ensureAtRepositoryRoot, ... }:
    let
      arion = inputs'.arion.packages.default;

      mkCosmosDevnet = import ./mkCosmosDevnet.nix {
        inherit pkgs dbg;
        ucliBin = pkgs.lib.getExe self'.packages.ucli;
      };
      lnav = inputs'.nixpkgs-lnav.legacyPackages.lnav;

      cosmwasmContracts = [
        {
          code = self'.packages.ucs00-pingpong;
          instances = [ ];
        }
        {
          code = self'.packages.ucs01-relay;
          instances = [ ];
        }
        {
          code = self'.packages.ucs02-nft;
          instances = [ ];
        }
        {
          code = self'.packages.cw721-base;
          instances = [ ];
        }
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
        cosmwasmContracts = [
          {
            code = self'.packages.ucs00-pingpong;
            instances = [ ];
          }
          {
            code = self'.packages.ucs01-relay;
            instances = [{
              message = {
                default_timeout = 10000;
                # Todo derive
                gov_contract = "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2";
              };
              # salt must be non-prefixed hex string
              salt = "00";
              label = "ucs01-relay";
            }];
          }
          {
            code = self'.packages.ucs02-nft;
            instances = [{
              message = {
                # Must be the index of `cw721-base` within this contracts list
                cw721_base_code_id = 4;
                incoming_proxy = null;
                outgoing_proxy = null;
                pauser = null;
                cw721_admin = null;
              };
              salt = "00";
              label = "ucs02-nft";
            }];
          }
          {
            code = self'.packages.cw721-base;
            instances = [ ];
          }
        ];
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
        inherit cosmwasmContracts;
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
        portIncrease = 300;
      };

      devnet-union-minimal = mkCosmosDevnet {
        node = (get-flake inputs.v0_21_0).packages.${system}.uniond;
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
          forge = import ./services/forge.nix {
            inherit pkgs;
            inherit (self'.packages) forge evm-sources;
          };
          lodestar = import ./services/lodestar.nix {
            inherit pkgs;
            config = self'.packages.devnet-eth-config;
            validatorCount = devnetConfig.ethereum.beacon.validatorCount;
          };
        }
        # For some reason, blockscout backend segfault on non-x86 arch
        // (if pkgs.stdenv.isx86_64 then {
          blockscout-backend = import ./services/blockscout/backend.nix {
            inherit lib pkgs;
            inherit (inputs) env-utils;
          };
          blockscout-frontend = import ./services/blockscout/frontend.nix {
            inherit lib pkgs;
            inherit (inputs) env-utils;
          };
          blockscout-sc-verifier = import ./services/blockscout/sc-verifier.nix {
            inherit lib pkgs;
            inherit (inputs) env-utils;
            inherit (self'.packages) evm-sources;
          };
          blockscout-db = import ./services/blockscout/db.nix {
            inherit lib pkgs;
          };
          blockscout-redis = import ./services/blockscout/redis.nix {
            inherit lib pkgs;
          };
          blockscout-sig-provider = import ./services/blockscout/sig-provider.nix {
            inherit lib pkgs;
          };
          blockscout-stats-db = import ./services/blockscout/stats-db.nix {
            inherit lib pkgs;
          };
          blockscout-stats = import ./services/blockscout/stats.nix {
            inherit lib pkgs;
          };
          blockscout-visualizer = import ./services/blockscout/visualizer.nix {
            inherit lib pkgs;
          };
          blockscout-proxy = import ./services/blockscout/proxy.nix {
            inherit lib pkgs;
          };
        } else { });

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
        devnet = pkgs.writeShellApplication
          {
            name = "union-full-devnet";
            runtimeInputs = [ pkgs.bash inputs'.process-compose.packages.process-compose ];
            text = ''
              ${ensureAtRepositoryRoot}

              rm -rf ./.devnet/homes/
              mkdir -p ./.devnet/homes/
              cp -R ${self'.packages.devnet-union-home} ./.devnet/homes/union/ 
              cp -R ${self'.packages.devnet-osmosis-home} ./.devnet/homes/osmosis/ 
              cp -R ${self'.packages.devnet-stargaze-home} ./.devnet/homes/stargaze/ 
              cp -R ${self'.packages.devnet-simd-home} ./.devnet/homes/simd/ 

              # Fix no write permission on keys
              chmod -R +w ./.devnet/homes

              ${lib.getExe self'.packages.devnet-compose}

              SHELL=${lib.getExe pkgs.bash} process-compose --theme="One Dark"
            '';
          };
        devnet-logs = pkgs.writeShellApplication
          {
            name = "union-full-devnet-logs";
            runtimeInputs = [ lnav ];
            text = ''
              ${ensureAtRepositoryRoot}
              lnav ./.devnet/logs/
            '';
          };
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
