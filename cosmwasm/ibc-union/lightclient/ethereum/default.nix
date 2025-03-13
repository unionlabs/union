_: {
  perSystem =
    {
      crane,
      ...
    }:
    let
      ethereum-light-client = crane.buildWorkspaceMember {
        crateDirFromRoot = "cosmwasm/ibc-union/lightclient/ethereum";
      };
    in
    {
      inherit (ethereum-light-client) checks;
    };
}
