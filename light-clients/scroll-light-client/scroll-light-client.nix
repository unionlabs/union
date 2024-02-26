{ ... }: {
  perSystem = { crane, lib, ensure-wasm-client-type, ... }:
    let
      workspace = (crane.buildWasmContract {
        crateDirFromRoot = "light-clients/scroll-light-client";
        checks = [
          (file_path: ''
            ${ensure-wasm-client-type {
              inherit file_path;
              type = "Scroll";
            }}
          '')
        ];
        additionalSrcFilter = path: _:
          (lib.hasPrefix "lib/poseidon-rs/constants.json" path);
        additionalTestSrcFilter = path: _:
          (lib.hasPrefix "light-clients/scroll-light-client/src/test" path)
          && (lib.strings.hasSuffix ".json" path);
      });
    in
    {
      inherit (workspace) packages checks;
    };
}
