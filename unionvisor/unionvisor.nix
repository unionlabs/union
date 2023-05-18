{ ... }: {
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

        bundle-testnet = mkBundle "testnet" [ "v0.0.2" "v0.3.0" "v0.4.2" ];
        bundle-mainnet = mkBundle "mainnet" [ "v0.0.2" "v0.3.0" ];
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
            ''cp ${self'.packages.uniond}/bin/uniond $PWD/src/testdata/test_init_cmd/bins/genesis && \
             echo "patching testdata" && \
             source ${pkgs.stdenv}/setup && patchShebangs $PWD/src/testdata
            ''
          ];
          buildPhase = ''
            cargo nextest run
          '';
          installPhase = ''
            mkdir -p $out
          '';
        });
      };
    };
}
