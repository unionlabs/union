{ ... }: {
  perSystem = { crane, lib, ensure-wasm-client-type, ... }:
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
        ];
        additionalTestSrcFilter = path: _:
          (lib.hasPrefix "light-clients/cometbls-light-client/src/test" path)
          && (lib.strings.hasSuffix ".json" path);
      });
    in
    {
      inherit (workspace) packages checks;
    };
}
