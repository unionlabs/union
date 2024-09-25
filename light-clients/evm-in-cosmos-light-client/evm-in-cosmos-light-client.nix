_: {
  perSystem =
    {
      crane,
      lib,
      ensure-wasm-client-type,
      ...
    }:
    let
      workspace = crane.buildWasmContract {
        crateDirFromRoot = "light-clients/evm-in-cosmos-light-client";
        checks = [
          (file_path: ''
            ${ensure-wasm-client-type {
              inherit file_path;
              type = "EvmInCosmos";
            }}
          '')
        ];
      };
    in
    {
      inherit (workspace) packages checks;
    };
}
