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
        crateDirFromRoot = "cosmwasm/union-ibc/light-clients/ethereum";
      };
    in
    {
      inherit (lc) packages;
      inherit (lc) checks;
    };
}
