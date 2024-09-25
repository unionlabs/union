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
      linea-zktrie-all = crane.buildWorkspaceMember {
        crateDirFromRoot = "lib/linea-zktrie";
      };
    in
    {
      inherit (linea-zktrie-all) checks;
    };
}
