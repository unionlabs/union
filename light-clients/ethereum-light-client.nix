{ ... }: {
  perSystem = { crane, ... }:
    {
      packages = {
        wasm-ethereum-lc = crane.buildWasmContract {
          cargoToml = ./ethereum-light-client/Cargo.toml;
          cargoLock = ../Cargo.lock;
          # features = [ "eth-minimal" ];
        };
      };
    };
}
