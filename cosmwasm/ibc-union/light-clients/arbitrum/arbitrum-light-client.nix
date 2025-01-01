_: {
  perSystem =
    {
      crane,
      lib,
      ...
    }:
    let
      workspace = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/ibc-union/light-clients/arbitrum";
      };
    in
    {
      inherit (workspace) packages checks;
    };
}
