{ ... }: {
  perSystem = { self', pkgs, system, config, crane, stdenv, dbg, ... }:
    let
      mkVoyager = features: pnameSuffix: (crane.buildWorkspaceMember {
        inherit pnameSuffix;
        crateDirFromRoot = "voyager";
        cargoBuildExtraArgs = features;
        additionalSrcFilter = path: _:
          pkgs.lib.hasPrefix ".sqlx" path;
        additionalTestSrcFilter = path: _:
          pkgs.lib.hasPrefix "hubble/src/graphql" path;
        extraEnv = {
          SQLX_OFFLINE = "1";
        };
      });

      voyagerMainnet = (mkVoyager "--features eth-mainnet" "-mainnet");
      voyagerMinimal = (mkVoyager "" "-minimal");
    in
    {
      packages = pkgs.lib.recursiveUpdate voyagerMainnet.packages voyagerMinimal.packages;
      checks = pkgs.lib.recursiveUpdate voyagerMainnet.checks voyagerMinimal.checks;
    };
}
