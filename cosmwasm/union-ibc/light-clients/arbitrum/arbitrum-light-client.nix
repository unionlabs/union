_: {
  perSystem =
    {
      crane,
      lib,
      ...
    }:
    let
      workspace = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/union-ibc/light-clients/arbitrum";
      };
    in
    {
      inherit (workspace) packages checks;
    };
}
