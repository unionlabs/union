{ ... }: {
  perSystem = { crane, lib, ensure-wasm-client-type, ... }:
    let
      ics08 = (crane.buildWasmContract {
        crateDirFromRoot = "light-clients/cometbls/ics08";
        checks = [
          (file_path: ''
            ${ensure-wasm-client-type {
              inherit file_path;
              type = "Cometbls";
            }}
          '')
        ];
      });
    in
    {
      inherit (ics08) packages checks;
    };
}
