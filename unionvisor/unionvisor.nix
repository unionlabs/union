{ self, inputs, ... }: {
  perSystem = { self', pkgs, system, config, inputs', crane, stdenv, get-flake, uniondBundleVersions, ... }:
    let
      swapDotsWithUnderscores = pkgs.lib.replaceStrings [ "." ] [ "_" ];

      unionvisorAll = crane.buildWorkspaceMember {
        crateDirFromRoot = "unionvisor";
        cargoTestExtraAttrs = {
          partitions = 1;
          partitionType = "count";
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
        };
      };

      mkBundle = { name, versions, genesis, meta, nextVersion ? "" }:
        pkgs.linkFarm "union-bundle-${name}" ([
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
            path = "${unionvisorAll.packages.unionvisor}/bin/unionvisor";
          }
        ]
        ++ # add all `versions` to the bundle
        map
          (version: {
            name =
              "${meta.versions_directory}/${version}/${meta.binary_name}";
            path = pkgs.lib.getExe (get-flake "${inputs."${swapDotsWithUnderscores version}"}").packages.${system}.uniond-release;
          })
          versions
        ++ # add `nextVersion` to the bundle if supplied
        pkgs.lib.lists.optional (nextVersion != "") ({
          name = "${meta.versions_directory}/${nextVersion}/${meta.binary_name}";
          path = pkgs.lib.getExe self'.packages.uniond-release;
        }));

      mkUnionvisorImage = unionvisorBundle: pkgs.dockerTools.buildImage {
        name = "unionvisor";
        copyToRoot = pkgs.buildEnv {
          name = "image-root";
          paths = [ pkgs.coreutils pkgs.cacert unionvisorBundle ];
          pathsToLink = [ "/bin" "/versions" "/" ];
        };
        config = {
          Entrypoint = [ "/unionvisor" ];
          Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" "UNIONVISOR_ROOT=/.unionvisor" "UNIONVISOR_BUNDLE=/" ];
        };
      };
    in
    {
      inherit (unionvisorAll) checks;
      packages = {
        inherit (unionvisorAll.packages) unionvisor;

        bundle-testnet-7-image = mkUnionvisorImage self'.packages.bundle-testnet-7;

        bundle-testnet-7 =
          mkBundle {
            name = "testnet-7";
            versions = uniondBundleVersions.complete;
            genesis = ../networks/genesis/union-testnet-7/genesis.json;
            meta = {
              binary_name = "uniond";
              versions_directory = "versions";
              fallback_version = uniondBundleVersions.first;
            };
          };

        bundle-testnet-next =
          mkBundle {
            name = "testnet-next";
            versions = uniondBundleVersions.complete;
            nextVersion = "v0.22.0";
            genesis = ../networks/genesis/union-testnet-7/genesis.json;
            meta = {
              binary_name = "uniond";
              versions_directory = "versions";
              fallback_version = uniondBundleVersions.first;
            };
          };
      };
    };

  flake.nixosModules.unionvisor = { lib, pkgs, config, ... }:
    with lib;
    let
      cfg = config.services.unionvisor;

      wrappedUnionvisor = pkgs.symlinkJoin {
        name = "unionvisor";
        paths = [ cfg.bundle ];
        buildInputs = [ pkgs.makeWrapper ];
        postBuild = ''
          wrapProgram $out/unionvisor \
            --set UNIONVISOR_ROOT /var/lib/unionvisor \
            --set HOME /var/lib/unionvisor \
            --set UNIONVISOR_BUNDLE ${cfg.bundle}

          mkdir -p $out/bin/
          mv $out/unionvisor $out/bin/unionvisor
        '';
      };
    in
    {
      options.services.unionvisor = {
        enable = mkEnableOption "Unionvisor service";
        bundle = mkOption {
          type = types.package;
          default = self.packages.${pkgs.system}.bundle-testnet-7;
        };
        logFormat = mkOption {
          type = types.str;
          default = "json";
          example = "plain";
        };
        moniker = mkOption { type = types.str; };
        network = mkOption {
          type = types.str;
          default = "union-testnet-6";
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
      };

      config = mkIf cfg.enable {
        environment.systemPackages = [
          wrappedUnionvisor
        ];

        systemd.services.unionvisor =
          let
            unionvisor-systemd-script = pkgs.writeShellApplication {
              name = "unionvisor-systemd";
              runtimeInputs = [ pkgs.coreutils wrappedUnionvisor ];
              text =
                let
                  configSymlinks = [
                    { name = "node_key.json"; path = cfg.node-key-json; }
                    { name = "priv_validator_key.json"; path = cfg.priv-validator-key-json; }
                    { name = "app.toml"; path = cfg.app-toml; }
                    { name = "client.toml"; path = cfg.client-toml; }
                    { name = "config.toml"; path = cfg.config-toml; }
                  ];

                  configSymLinkCommands = pkgs.lib.concatMapStrings
                    (l:
                      ''

                        rm ./home/config/${l.name}
                        ln -s ${l.path} ./home/config/${l.name}

                      '')
                    (builtins.filter (l: l.path != null) configSymlinks);

                in
                ''
                  ${pkgs.coreutils}/bin/mkdir -p /var/lib/unionvisor
                  cd /var/lib/unionvisor
                  unionvisor --log-format ${cfg.logFormat} init --moniker ${cfg.moniker} --seeds ${cfg.seeds} --network ${cfg.network} --allow-dirty}

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
          };
      };
    };
}



