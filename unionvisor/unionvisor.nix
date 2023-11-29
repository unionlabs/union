{ self, inputs, ... }: {
  perSystem = { self', pkgs, system, config, inputs', crane, stdenv, get-flake, uniondBundleVersions, ... }:
    let
      swapDotsWithUnderscores = pkgs.lib.replaceStrings [ "." ] [ "_" ];

      unionvisorAll = crane.buildWorkspaceMember {
        crateDirFromRoot = "unionvisor";
        additionalTestSrcFilter = path: _type:
          pkgs.lib.hasPrefix "unionvisor/src/testdata/" path;
        cargoTestExtraAttrs = {
          partitions = 1;
          partitionType = "count";
          preConfigureHooks = [
            ''
              cp -r ${self'.packages.uniond}/bin/uniond $PWD/unionvisor/src/testdata/test_init_cmd/bundle/bins/genesis
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
            path = pkgs.lib.getExe (get-flake "${inputs."${swapDotsWithUnderscores version}"}").packages.${system}.uniond;
          })
          versions
        ++ # add `nextVersion` to the bundle if supplied
        pkgs.lib.lists.optional (nextVersion != "") ({
          name = "${meta.versions_directory}/${nextVersion}/${meta.binary_name}";
          path = pkgs.lib.getExe self'.packages.uniond;
        }));

      mkUnionvisorImage = unionvisorBundle: pkgs.dockerTools.buildImage {
        name = "uniond";
        copyToRoot = pkgs.buildEnv {
          name = "image-root";
          paths = [ pkgs.coreutils pkgs.cacert unionvisorBundle ];
          pathsToLink = [ "/bin" ];
        };
        config = {
          Entrypoint = [ "unionvisor" ];
          Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
        };
      };
    in
    {
      inherit (unionvisorAll) checks;
      packages = {
        inherit (unionvisorAll.packages) unionvisor;

        bundle-testnet-4-image = mkUnionvisorImage self'.packages.bundle-testnet-4;

        bundle-testnet-4 =
          mkBundle {
            name = "testnet-4";
            versions = uniondBundleVersions.complete;
            genesis = ../networks/genesis/union-testnet-4/genesis.json;
            meta = {
              binary_name = "uniond";
              versions_directory = "versions";
              fallback_version = uniondBundleVersions.first;
            };
          };

        bundle-testnet-next =
          mkBundle {
            name = "testnet-4";
            versions = uniondBundleVersions.complete;
            genesis = ../networks/genesis/union-testnet-4/genesis.json;
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
          default = self.packages.${pkgs.system}.bundle-testnet-4;
        };
        moniker = mkOption { type = types.str; };
        network = mkOption {
          type = types.str;
          default = "union-testnet-4";
        };
        seeds = mkOption {
          type = types.str;
          default = "b4d587b3d3666d52df0cd43962080fd164568fe0@union-testnet.cor.systems:26656";
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
              text = ''
                ${pkgs.coreutils}/bin/mkdir -p /var/lib/unionvisor
                cd /var/lib/unionvisor
                unionvisor init  --moniker ${cfg.moniker} --seeds ${cfg.seeds} --network ${cfg.network} --allow-dirty
                unionvisor run
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
