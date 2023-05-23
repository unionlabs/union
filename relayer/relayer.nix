{ ... }: {
  perSystem = { self', pkgs, system, config, inputs', crane, stdenv, ... }:
    let
      attrs = crane.commonAttrs // {
        inherit (crane) cargoArtifacts;
        cargoExtraArgs = "-p relayer";
      } // (crane.lib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; });
    in
    {
      packages.relayer = crane.lib.buildPackage attrs;

      checks = crane.mkChecks "relayer" {
        # Temporarily commented out while in POC phase
        # clippy = crane.lib.cargoClippy ((builtins.trace attrs attrs) // {
        #   cargoClippyExtraArgs = "-- --deny warnings --no-deps";
        # });

        tests = crane.lib.cargoNextest attrs;
      };
    };
}

