{ ... }: {
  perSystem = { crane, lib, ensure-wasm-client-type, ensure-wasm-client-size, ... }:
    let
      workspace = (crane.buildWasmContract {
        crateDirFromRoot = "light-clients/cometbls-light-client";
        checks = [
          (file_path: ''
            ${ensure-wasm-client-type {
              inherit file_path;
              type = "Cometbls";
            }}
          '')
          (file_path: ''
            ${ensure-wasm-client-size {
              inherit file_path;
              max_size = 800 * 1024;
            }}
          '')
        ];
      });
    in
    {
      inherit (workspace) packages checks;
    };
}
