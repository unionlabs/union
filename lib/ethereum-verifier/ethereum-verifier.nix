_: {
  perSystem =
    {
      self',
      pkgs,
      system,
      config,
      crane,
      stdenv,
      dbg,
      lib,
      ...
    }:
    let
      ethereum-verifier-all = crane.buildWorkspaceMember {
        crateDirFromRoot = "lib/ethereum-verifier";
      };
    in
    {
      inherit (ethereum-verifier-all) checks;
    };
}
