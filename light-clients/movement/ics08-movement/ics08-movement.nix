_: {
  perSystem =
    {
      crane,
      lib,
      ensure-wasm-client-type,
      ...
    }:
    let
      lc = crane.buildWasmContract {
        crateDirFromRoot = "light-clients/movement/ics08-movement";
        checks = [
          (file_path: ''
            ${ensure-wasm-client-type {
              inherit file_path;
              type = "Movement";
            }}
          '')
        ];
      };
    in
    {
      inherit (lc) packages checks;
    };
}
