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
      arbitrum-verifier-all = crane.buildWorkspaceMember {
        crateDirFromRoot = "lib/arbitrum-verifier";
      };
    in
    {
      inherit (arbitrum-verifier-all) checks;
    };
}
