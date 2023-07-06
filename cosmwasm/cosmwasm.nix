{ ... }: {
  perSystem = { crane, ... }:
    {
      packages = {
        wasm-cw20-ics20 = crane.buildWasmContract {
          cargoToml = ./cw20-ics20/Cargo.toml;
          cargoLock = ../Cargo.lock;
        };
        wasm-ucs00-pingpong = crane.buildWasmContract {
          cargoToml = ./ucs00-pingpong/Cargo.toml;
          cargoLock = ../Cargo.lock;
        };
      };
    };
}
