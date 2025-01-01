_: {
  perSystem =
    {
      crane,
      lib,
      ...
    }:
    let
      lc = crane.buildWasmContract {
        crateDirFromRoot = "cosmwasm/union-ibc/light-clients/evm-in-cosmos";
      };
    in
    {
      inherit (lc) packages checks;
    };
}
