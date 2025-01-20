_: {
  perSystem =
    {
      crane,
      lib,
      ...
    }:
    let
      lc = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/ibc-union/light-clients/state-lens-ics23-mpt";
      };
    in
    {
      inherit (lc) packages checks;
    };
}
