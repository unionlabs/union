_: {
  perSystem =
    {
      crane,
      lib,
      ...
    }:
    let
      lc = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/ibc-union/light-clients/tendermint";
      };
    in
    {
      inherit (lc) packages checks;
    };
}
