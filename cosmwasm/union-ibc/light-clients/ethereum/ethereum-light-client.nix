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
      };
    in
    {
      inherit (lc) packages;
      inherit (lc) checks;
    };
}
