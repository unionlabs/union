{ ... }: {
  perSystem = { self', pkgs, system, config, inputs', crane, stdenv, ... }:
    let
      attrs = crane.commonAttrs // {
        inherit (crane) cargoArtifacts;
        cargoExtraArgs = "-p relayer";
      } // (crane.lib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; });

      relayer = crane.lib.buildPackage attrs;
    in
    {
      packages = {
        inherit relayer;
      };

      checks = crane.mkChecks "relayer" {
        # clippy = crane.lib.cargoClippy ((builtins.trace attrs attrs) // {
        #   cargoClippyExtraArgs = "-- --deny warnings --no-deps";
        # });

        tests = crane.lib.cargoNextest attrs;
      };
    };
}

