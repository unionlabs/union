_: {
  perSystem =
    {
      crane,
      lib,
      ...
    }:
    let
      lc = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/ibc-union/light-clients/evm-in-cosmos";
      };
    in
    {
      inherit (lc) packages checks;
    };
}
