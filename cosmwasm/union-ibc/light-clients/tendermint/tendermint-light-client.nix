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
        crateDirFromRoot = "cosmwasm/union-ibc/light-clients/tendermint";
        checks = [
          (file_path: ''
            ${ensure-wasm-client-type {
              inherit file_path;
              type = "Tendermint";
            }}
          '')
        ];
      };
    in
    {
      inherit (workspace) packages checks;
    };
}
