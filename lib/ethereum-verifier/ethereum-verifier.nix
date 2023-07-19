{ ... }: {
  perSystem = { self', pkgs, system, config, inputs', crane, stdenv, ... }:
    let
      attrs = crane.commonAttrs // {
        inherit (crane) cargoArtifacts;
        cargoExtraArgs = "-p ethereum-verifier";
      } // (crane.lib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; });
    in
    {

      checks = crane.mkChecks "ethereum-verifier" {
        clippy = crane.lib.cargoClippy (attrs // {
          cargoClippyExtraArgs = "-- --deny warnings --no-deps";
        });

        tests = crane.lib.cargoNextest attrs;
      };
    };
}

