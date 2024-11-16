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
        crateDirFromRoot = "light-clients/berachain-light-client";
        checks = [
          (file_path: ''
            ${ensure-wasm-client-type {
              inherit file_path;
              type = "Berachain";
            }}
          '')
        ];
      };
    in
    {
      inherit (workspace) packages checks;
    };
}
