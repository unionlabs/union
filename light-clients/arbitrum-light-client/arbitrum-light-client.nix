{ ... }: {
  perSystem = { crane, lib, ensure-wasm-client-type, ... }:
    let
      workspace = (crane.buildWasmContract {
        crateDirFromRoot = "light-clients/arbitrum-light-client";
        checks = [
          (file_path: ''
            ${ensure-wasm-client-type {
              inherit file_path;
              type = "Arbitrum";
            }}
          '')
        ];
      });
    in
    {
      inherit (workspace) packages checks;
    };
}
