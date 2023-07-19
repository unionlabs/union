{ ... }: {
  perSystem = { crane, lib, ... }:
    let
      attrs = crane.commonAttrs // {
        inherit (crane) cargoArtifacts;
        cargoExtraArgs = "-p ethereum-light-client --features minimal";
      } // (crane.lib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; });
    in
    {
      packages =
        lib.listToAttrs (map
          (config: lib.nameValuePair "ethereum-light-client-${config}" (crane.buildWasmContract {
            cargoToml = ./Cargo.toml;
            cargoLock = ../../Cargo.lock;
            features = [ config ];
          })) [ "mainnet" "minimal" ]);

      checks = crane.mkChecks "ethereum-light-client" {
        clippy = crane.lib.cargoClippy (attrs // {
          cargoClippyExtraArgs = "-- --deny warnings --no-deps";
        });

        tests = crane.lib.cargoNextest attrs;
      };
    };
}
