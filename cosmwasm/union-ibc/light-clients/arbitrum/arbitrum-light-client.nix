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
        crateDirFromRoot = "cosmwasm/union-ibc/light-clients/arbitrum";
        checks = [
          (file_path: ''
            ${ensure-wasm-client-type {
              inherit file_path;
              type = "Arbitrum";
            }}
          '')
        ];
      };
    in
    {
      inherit (workspace) packages checks;
    };
}
