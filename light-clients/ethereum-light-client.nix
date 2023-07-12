{ ... }: {
  perSystem = { crane, lib, ... }:
    {
      packages =
        lib.listToAttrs (map
          (config: lib.nameValuePair "ethereum-light-client-${config}" (crane.buildWasmContract {
            cargoToml = ./ethereum-light-client/Cargo.toml;
            cargoLock = ../Cargo.lock;
            features = [ config ];
          })) [ "mainnet" "minimal" ]);
    };
}
