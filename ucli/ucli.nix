{ ... }: {
  perSystem = { self', pkgs, crane, ... }:
    let
      ucli = crane.buildWorkspaceMember {
        crateDirFromRoot = "ucli";
        additionalTestSrcFilter = path: _:
          pkgs.lib.hasPrefix "hubble/src/graphql" path;
        cargoTestExtraAttrs = {
          partitions = 1;
          partitionType = "count";
        };
      };
    in
    {
      packages = ucli.packages;
      checks = ucli.checks;
    };
}
