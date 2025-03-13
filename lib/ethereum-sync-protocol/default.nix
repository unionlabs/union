_: {
  perSystem =
    {
      crane,
      ...
    }:
    let
      ethereum-sync-protocol = crane.buildWorkspaceMember {
        crateDirFromRoot = "lib/ethereum-sync-protocol";
      };
    in
    {
      inherit (ethereum-sync-protocol) checks;
    };
}
