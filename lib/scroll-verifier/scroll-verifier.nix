{ ... }: {
  perSystem = { self', pkgs, system, config, crane, stdenv, dbg, lib, ... }:
    let
      scroll-verifier-all = (crane.buildWorkspaceMember {
        crateDirFromRoot = "lib/scroll-verifier";
        additionalTestSrcFilter = path: _:
          (lib.hasPrefix "lib/scroll-verifier/tests" path)
          && (lib.strings.hasSuffix ".json" path);
      });
    in
    {
      inherit (scroll-verifier-all) checks;
    };
}
