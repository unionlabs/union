{ ... }: {
  perSystem = { crane, lib, ensure-wasm-client-type, ... }:
    let
      workspace = (crane.buildWasmContract {
        crateDirFromRoot = "light-clients/linea-light-client";
        checks = [
          (file_path: ''
            ${ensure-wasm-client-type {
              inherit file_path;
              type = "Linea";
            }}
          '')
        ];
      });
    in
    {
      inherit (workspace) packages checks;
    };
}
