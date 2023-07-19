{ ... }: {
  perSystem = { self', pkgs, system, config, inputs', crane, stdenv, ... }:
    let
      attrs = crane.commonAttrs // {
        inherit (crane) cargoArtifacts;
        cargoExtraArgs = "-p e2e";
      } // (crane.lib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; });
    in
    {
      packages.e2e = crane.lib.buildPackage attrs;
    };
}

