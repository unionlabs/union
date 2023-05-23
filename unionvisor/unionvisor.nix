{ self, ... }: {
  perSystem = { self', pkgs, system, config, inputs', crane, stdenv, ... }:
    let
      attrs = crane.commonAttrs // {
        inherit (crane) cargoArtifacts;
        cargoExtraArgs = "-p unionvisor";
      } // (crane.lib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; });

      unionvisor = crane.lib.buildPackage attrs;

      mkBundle = name: versions: meta: pkgs.linkFarm "union-bundle-${name}" ([
        {
          name = "meta.json";
          path = pkgs.writeText "meta.json" (builtins.toJSON meta);
        }
        {
          name = "unionvisor";
          path = "${unionvisor}/bin/unionvisor";
        }
      ] ++ map
        (version: {
          name = "${meta.versions_directory}/${version}/${meta.binary_name}";
          path = pkgs.lib.getExe inputs'."${version}".packages.uniond;
        })
        versions);
    in
    {
      packages = {
        inherit unionvisor;

        bundle-testnet = mkBundle "testnet" [ "v0.2.0" "v0.5.0" ] {
          binary_name = "uniond";
          versions_directory = "versions";
          fallback_version = "v0.2.0";
        };
        bundle-mainnet = mkBundle "mainnet" [ "v0.2.0" "v0.5.0" ] {
          binary_name = "uniond";
          versions_directory = "versions";
          fallback_version = "v0.2.0";
        };
      };

      checks = crane.mkChecks "unionvisor" {
        clippy = crane.lib.cargoClippy ((builtins.trace attrs attrs) // {
          cargoClippyExtraArgs = "-- --deny warnings --no-deps";
        });

        tests = crane.lib.cargoNextest (attrs // {
          inherit (crane) cargoArtifacts;
          partitions = 1;
          partitionType = "count";
          preConfigureHooks = [
            ''cp ${self'.packages.uniond}/bin/uniond $PWD/unionvisor/src/testdata/test_init_cmd/bundle/bins/genesis && \
             echo "patching testdata" && \
             source ${pkgs.stdenv}/setup && patchShebangs $PWD/unionvisor/src/testdata
            ''
          ];
          buildPhase = ''
            cargo nextest run -p unionvisor
          '';
          installPhase = ''
            mkdir -p $out
          '';
        });
      };
    };

  flake.nixosModules.unionvisor = { lib, pkgs, config, ... }:
    with lib;
    let
      cfg = config.services.unionvisor;
    in
    {
      options.services.unionvisor = {
        enable = mkEnableOption "Unionvisor service";
        bundle = mkOption {
          type = types.package;
          default = self.packages.${pkgs.system}.bundle-testnet;
        };
        moniker = mkOption {
          type = types.str;
        };
      };

      config = mkIf cfg.enable {
        systemd.services.unionvisor =
          let
            unionvisor-systemd-script = pkgs.writeShellApplication {
              name = "unionvisor-systemd";
              runtimeInputs = [ pkgs.coreutils cfg.bundle ];
              text = ''
                ${pkgs.coreutils}/bin/mkdir -p /var/lib/unionvisor 
                cd /var/lib/unionvisor 
                HOME=/var/lib/unionvisor ${cfg.bundle}/unionvisor --root /var/lib/unionvisor init --bundle ${cfg.bundle} --moniker ${cfg.moniker} --allow-dirty 
                HOME=/var/lib/unionvisor ${cfg.bundle}/unionvisor --root /var/lib/unionvisor run --bundle ${cfg.bundle} 
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
