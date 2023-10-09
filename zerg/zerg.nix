{ ... }: {
  perSystem = { self', pkgs, system, config, crane, stdenv, ... }:
    let
      mkZerg = features: pnameSuffix: (crane.buildWorkspaceMember {
        inherit pnameSuffix;
        crateDirFromRoot = "zerg";
        cargoBuildExtraArgs = features;
      });

      zerg = (mkZerg "" "");
    in
    {
      packages = zerg.packages;
    };
}
