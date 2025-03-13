_: {
  perSystem =
    {
      crane,
      ...
    }:
    let
      evm-storage-verifier = crane.buildWorkspaceMember {
        crateDirFromRoot = "lib/evm-storage-verifier";
      };
    in
    {
      inherit (evm-storage-verifier) checks;
    };
}
