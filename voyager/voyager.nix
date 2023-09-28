{ ... }: {
  perSystem = { self', pkgs, system, config, crane, stdenv, ... }:
    let
      voyagerAll = (crane.buildWorkspaceMember {
        crateDirFromRoot = "voyager";
        additionalTestSrcFilter = path: _:
          (pkgs.lib.hasPrefix "hubble/src/graphql" path);
      });
    in
    {
      inherit (voyagerAll) packages checks;
    };
}
