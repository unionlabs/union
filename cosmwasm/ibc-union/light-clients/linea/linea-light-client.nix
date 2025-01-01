_: {
  perSystem =
    {
      crane,
      lib,
      ...
    }:
    let
      lc = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/ibc-union/light-clients/linea";
      };
    in
    {
      inherit (lc) packages checks;
    };
}
