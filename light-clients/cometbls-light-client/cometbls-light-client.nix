{ ... }: {
  perSystem = { crane, lib, ... }:
    let
      attrs = crane.commonAttrs // {
        inherit (crane) cargoArtifacts;
        cargoExtraArgs = "-p ethereum-light-client --features minimal";
      } // (crane.lib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; });
    in
    {
      packages = {
        wasm-cometbls-light-client = crane.buildWasmContract {
          cargoToml = ./Cargo.toml;
          cargoLock = ../../Cargo.lock;
        };
      };

      checks = crane.mkChecks "cometbls-light-client" {
        clippy = crane.lib.cargoClippy (attrs // {
          cargoClippyExtraArgs = "-- --deny warnings --no-deps";
        });

        tests = crane.lib.cargoNextest attrs;
      };
    };
}
