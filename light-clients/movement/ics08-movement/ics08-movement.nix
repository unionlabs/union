{ ... }: {
  perSystem = { crane, lib, ensure-wasm-client-type, ... }:
    let
      workspace = (crane.buildWasmContract {
        crateDirFromRoot = "light-clients/movement/ics08-movement";
        checks = [
          (file_path: ''
            ${ensure-wasm-client-type {
              inherit file_path;
              type = "Movement";
            }}
          '')
        ];
      });
    in
    {
      inherit (workspace) packages checks;
    };
}
