{ ... }: {
  perSystem = { self', pkgs, system, config, crane, stdenv, dbg, ... }:
    let
      voyager = crane.buildWorkspaceMember {
        crateDirFromRoot = "voyager";
        additionalSrcFilter = path: _:
          pkgs.lib.hasPrefix ".sqlx" path;
        additionalTestSrcFilter = path: _:
          pkgs.lib.hasPrefix "hubble/src/graphql" path;
        # temporarily, to keep warnings in-editor until i fix them
        cargoClippyExtraArgs = "--allow deprecated";
        extraEnv = {
          SQLX_OFFLINE = "1";
        };
      };
    in
    {
      packages = voyager.packages;
      checks = voyager.checks;
    };
}
