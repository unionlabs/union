{ ... }: {
  perSystem = { crane, ... }:
    {
      packages = {
        wasm-ethereum-lc = crane.buildWasmContract {
          cargoToml = ./ethereum-light-client/Cargo.toml;
          cargoLock = ../Cargo.lock;
          # TODO(aeryz): remove this before merging, and instead, create an another derivation for minimal config
          features = [ "eth-minimal" ];
        };
      };
    };
}
