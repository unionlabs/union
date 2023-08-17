{ self, ... }: {
  perSystem = { self', pkgs, system, config, inputs', crane, stdenv, ... }:
    let
      attrs = crane.commonAttrs // {
        inherit (crane) cargoArtifacts;
        cargoExtraArgs = "-p hubble";
      } // (crane.lib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; });

      hubble = crane.lib.buildPackage attrs;
    in
    {
      packages = {
        inherit hubble;
      };

      checks = crane.mkChecks "hubble" {
        clippy = crane.lib.cargoClippy ((builtins.trace attrs attrs) // {
          cargoClippyExtraArgs = "-- --deny warnings --no-deps";
        });

        tests = crane.lib.cargoNextest (attrs // {
          inherit (crane) cargoArtifacts;
          partitions = 1;
          partitionType = "count";
          buildPhase = ''
            cargo nextest run -p hubble
          '';
          installPhase = ''
            mkdir -p $out
          '';
        });
      };
    };
}
