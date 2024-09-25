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
      scroll-verifier-all = crane.buildWorkspaceMember {
        crateDirFromRoot = "lib/scroll-verifier";
      };
    in
    {
      inherit (scroll-verifier-all) checks;
    };
}
