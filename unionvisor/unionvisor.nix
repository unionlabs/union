{ self, ... }: {
  perSystem = { self', pkgs, system, config, inputs', crane, stdenv, ... }:
    let
      attrs = crane.commonAttrs // {
        inherit (crane) cargoArtifacts;
        cargoExtraArgs = "-p unionvisor";
      } // (crane.lib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; });

      unionvisor = crane.lib.buildPackage attrs;

      mkBundle = name: versions: pkgs.linkFarm "union-bundle-${name}" ([
        {
          name = "unionvisor";
          path = "${unionvisor}/bin/unionvisor";
        }
      ] ++ map
        (version: {
          name = "bins/${version}/uniond";
          path = "${inputs'."${version}".packages.uniond}/bin/uniond";
        })
        versions);
    in
    {
      packages = {
        inherit unionvisor;

        bundle-testnet = mkBundle "testnet" [ "v0.5.0" ];
        bundle-mainnet = mkBundle "mainnet" [ "v0.5.0" ];
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
            ''cp ${self'.packages.uniond}/bin/uniond $PWD/unionvisor/src/testdata/test_init_cmd/bins/genesis && \
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
        monniker = mkOption {
          type = types.str;
        };
      };

      config = mkIf cfg.enable {
        systemd.services.unionvisor = {
          wantedBy = [ "multi-user.target" ];
          description = "Unionvisor";
          serviceConfig = {
            Type = "simple";
            WorkingDirectory = "/home/unionvisor";
            ExecStart = "${cfg.bundle}/bin/unionvisor init --bindir ${cfg.bundle}/bins --monniker ${cfg.monniker}";
            Restart = mkForce "always";
          }; 
        };
      };
    };
}
