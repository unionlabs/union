{ ... }: {
  perSystem = { self', pkgs, crane, ... }:
    let
      ucli = crane.buildWorkspaceMember {
        crateDirFromRoot = "ucli";
        additionalSrcFilter = path: _type: pkgs.lib.hasPrefix "hubble/src/graphql/" path || pkgs.lib.hasPrefix ".sqlx" path;
        additionalTestSrcFilter = path: _:
          pkgs.lib.hasPrefix "hubble/src/graphql" path;
        cargoTestExtraAttrs = {
          partitions = 1;
          partitionType = "count";
        };
        extraEnv = {
          SQLX_OFFLINE = "1";
        };
      };
    in
    {
      packages = ucli.packages;
      checks = ucli.checks;
    };
}
