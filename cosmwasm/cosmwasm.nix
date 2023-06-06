{ ... }: {
  perSystem = { crane, ... }:
    {
      packages = {
        wasm-cw20-ics20 = crane.buildWasmContract {
          cargoToml = ./cw20-ics20/Cargo.toml;
          cargoLock = ../Cargo.lock;
        };
      };
    };
}
