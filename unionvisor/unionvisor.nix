{ self, inputs, ... }:
{
  perSystem =
    {
      self',
      pkgs,
      system,
      config,
      inputs',
      crane,
      stdenv,
      get-flake,
      uniondBundleVersions,
      dbg,
      ...
    }:
    let
      swapDotsWithUnderscores = pkgs.lib.replaceStrings [ "." ] [ "_" ];

      unionvisor = crane.buildWorkspaceMember "unionvisor" {
        dontRemoveDevDeps = true;
      };

      mkBundle =
        {
          name,
          versions,
          genesis,
          meta,
          nextVersion ? "",
        }:
        pkgs.linkFarm "union-bundle-${name}" (
          [
            {
              name = "meta.json";
              path = pkgs.writeText "meta.json" (builtins.toJSON meta);
            }
            {
              name = "genesis.json";
              path = genesis;
            }
            {
              name = "unionvisor";
              path = "${unionvisor.unionvisor}/bin/unionvisor";
            }
          ]
          # add all `versions` to the bundle
          ++ map (version: {
            name = "${meta.versions_directory}/${version}/${meta.binary_name}";
            # Dynamically load the flake dependency to avoid having the full tree in the lock file.
            path =
              pkgs.lib.getExe
                (get-flake "${inputs."${swapDotsWithUnderscores version}"}").packages.${system}.uniond-release;
          }) versions
          # add `nextVersion` to the bundle if supplied
          ++ pkgs.lib.lists.optional (nextVersion != "") {
            name = "${meta.versions_directory}/${nextVersion}/${meta.binary_name}";
            path = pkgs.lib.getExe self'.packages.uniond-release;
          }
        );

      mkUnionvisorImage =
        unionvisorBundle:
        pkgs.dockerTools.buildImage {
          name = "unionvisor";
          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            paths = [
              pkgs.coreutils
              pkgs.cacert
              unionvisorBundle
            ];
            pathsToLink = [
              "/bin"
              "/versions"
              "/"
            ];
          };
          config = {
            Entrypoint = [ "/unionvisor" ];
            Env = [
              "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
              "UNIONVISOR_ROOT=/.unionvisor"
              "UNIONVISOR_BUNDLE=/"
            ];
          };
        };
    in
    {
      checks.unionvisor-tests = crane.lib.cargoTest (
        unionvisor.unionvisor.passthru.craneAttrs
        // {
          doCheck = true;
          cargoTestExtraArgs = "-- --test-threads 1";
          preConfigureHooks = [
            ''
              cp -r ${self'.packages.uniond-release}/bin/uniond $PWD/unionvisor/src/testdata/test_init_cmd/bundle/bins/genesis
            ''
            ''
              echo 'patching testdata'
            ''
            ''
              patchShebangs $PWD/unionvisor/src/testdata
            ''
          ];
        }
      );
      packages = unionvisor // {
        bundle-union-1-image = mkUnionvisorImage self'.packages.bundle-union-1;

        bundle-union-1 = mkBundle {
          name = "union-1";
          versions = uniondBundleVersions.complete;
          genesis = ../networks/genesis/union-testnet-9/genesis.json;
          meta = {
            binary_name = "uniond";
            versions_directory = "versions";
            fallback_version = uniondBundleVersions.first;
          };
        };

        bundle-union-testnet-10-image = mkUnionvisorImage self'.packages.bundle-union-testnet-10;

        bundle-union-testnet-10 = mkBundle {
          name = "union-testnet-10";
          versions = uniondBundleVersions.complete;
          genesis = ../networks/genesis/union-testnet-10/genesis.json;
          meta = {
            binary_name = "uniond";
            versions_directory = "versions";
            fallback_version = uniondBundleVersions.first;
          };
        };

        bundle-union-1-next = mkBundle {
          name = "union-1-next";
          versions = uniondBundleVersions.complete;
          nextVersion = "v1.1.0";
          genesis = ../networks/genesis/union-testnet-9/genesis.json;
          meta = {
            binary_name = "uniond";
            versions_directory = "versions";
            fallback_version = uniondBundleVersions.first;
          };
        };
      };
    };

  flake.nixosModules.unionvisor =
    {
      lib,
      pkgs,
      config,
      ...
    }:
    with lib;
    let
      cfg = config.services.unionvisor;
    in
    {
      options.services.unionvisor = {
        enable = mkEnableOption "Unionvisor service";
        package = mkOption {
          type = types.package;
          default = self.packages.${pkgs.system}.unionvisor;
        };
        bundle = mkOption {
          type = types.package;
          default = self.packages.${pkgs.system}.bundle-union-1;
        };
        logFormat = mkOption {
          type = types.enum [
            "json"
            "plain"
          ];
          default = "json";
          example = "plain";
        };
        moniker = mkOption { type = types.str; };
        network = mkOption {
          type = types.str;
          default = "union-1";
        };
        seeds = mkOption {
          type = types.str;
          default = "b4d587b3d3666d52df0cd43962080fd164568fe0@union-testnet.cor.systems:26656";
        };
        node-key-json = mkOption {
          description = lib.mdDoc ''
            Path to a node_key.json file.
          '';
          example = "/run/secrets/node_key.json";
          type = types.nullOr types.path;
          default = null;
        };
        priv-validator-key-json = mkOption {
          description = lib.mdDoc ''
            Path to a priv_validator_key.json file.
          '';
          example = "/run/secrets/priv_validator_key.json";
          type = types.nullOr types.path;
          default = null;
        };
        app-toml = mkOption {
          description = lib.mdDoc ''
            Path to an app.toml file.
          '';
          example = "/some/app.toml";
          type = types.nullOr types.path;
          default = null;
        };
        config-toml = mkOption {
          description = lib.mdDoc ''
            Path to an config.toml file.
          '';
          example = "/some/config.toml";
          type = types.nullOr types.path;
          default = null;
        };
        client-toml = mkOption {
          description = lib.mdDoc ''
            Path to an client.toml file.
          '';
          example = "/some/client.toml";
          type = types.nullOr types.path;
          default = null;
        };
        root = mkOption {
          type = types.str;
          default = "/var/lib/unionvisor";
        };
        home = mkOption {
          type = types.str;
          default = "/var/lib/unionvisor";
        };
        extra-args = mkOption {
          description = lib.mdDoc ''
            Extra arguments to unionvisor.
          '';
          type = types.listOf types.str;
          default = [ ];
        };
      };

      config = mkIf cfg.enable {
        environment.systemPackages = [
          cfg.package
        ];

        systemd.services.unionvisor =
          let
            unionvisor-systemd-script = pkgs.writeShellApplication {
              name = "unionvisor-systemd";
              runtimeInputs = [
                pkgs.coreutils
                cfg.package
              ];
              text =
                let
                  configSymlinks = [
                    {
                      name = "node_key.json";
                      path = cfg.node-key-json;
                    }
                    {
                      name = "priv_validator_key.json";
                      path = cfg.priv-validator-key-json;
                    }
                    {
                      name = "app.toml";
                      path = cfg.app-toml;
                    }
                    {
                      name = "client.toml";
                      path = cfg.client-toml;
                    }
                    {
                      name = "config.toml";
                      path = cfg.config-toml;
                    }
                  ];

                  configSymLinkCommands = pkgs.lib.concatMapStrings (l: ''
                    export UNIONVISOR_BUNDLE="${cfg.bundle}"
                    export UNIONVISOR_ROOT="${cfg.root}"
                    export HOME="${cfg.home}"

                    cd "${cfg.root}"

                    pwd
                    rm ./home/config/${l.name}
                    ln -s ${l.path} ./home/config/${l.name}

                  '') (builtins.filter (l: l.path != null) configSymlinks);

                in
                ''
                  ${pkgs.coreutils}/bin/mkdir -p /var/lib/unionvisor
                  cd /var/lib/unionvisor
                  unionvisor --log-format ${cfg.logFormat} init --moniker ${cfg.moniker} --seeds ${cfg.seeds} --network ${cfg.network} --allow-dirty ${builtins.concatStringsSep " " cfg.extra-args}

                  ${configSymLinkCommands}

                  unionvisor --log-format ${cfg.logFormat} run
                '';
            };
          in
          {
            wantedBy = [ "multi-user.target" ];
            description = "Unionvisor";
            serviceConfig = {
              Type = "simple";
              ExecStart = pkgs.lib.getExe unionvisor-systemd-script;
              Restart = mkForce "always";
            };
            environment = {
              UNIONVISOR_BUNDLE = cfg.bundle;
              UNIONVISOR_ROOT = cfg.root;
              HOME = cfg.home;
            };
          };
      };
    };
}
