_: {
  perSystem =
    {
      crane,
      lib,
      dbg,
      pkgs,
      ensure-wasm-client-type,
      mkCi,
      ...
    }:
    let
      lc = crane.buildWasmContract {
        crateDirFromRoot = "light-clients/ethereum-light-client";
        checks = [
          (file_path: ''
            ${ensure-wasm-client-type {
              inherit file_path;
              type = "Ethereum";
            }}
          '')
        ];
      };
    in
    {
      inherit (lc) packages;
      inherit (lc) checks;
    };
}
