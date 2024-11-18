_: {
  perSystem =
    {
      crane,
      lib,
      ...
    }:
    let
      lc = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/union-ibc/light-clients/linea";
      };
    in
    {
      inherit (lc) packages checks;
    };
}
