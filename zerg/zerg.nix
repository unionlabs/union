{ ... }:
{
  perSystem = { self', pkgs, system, config, crane, stdenv }:
    let
      mkZerg = features: pnameSuffix: (crane.buildWorkspaceMember {
        inherit pnameSuffix;
        createDirFromRoot = "zerg";
        cargoBuildExtraArgs = features;
      });

      zerg = (mkZerg "" "");
    in
    {
      packages = pkgs.lib.recursiveUpdate zerg.packages;
    };
}
