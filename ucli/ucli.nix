{ ... }: {
  perSystem = { self', pkgs, crane, ... }:
    let
      mkUcli = features: pnameSuffix: (crane.buildWorkspaceMember {
        inherit pnameSuffix;
        crateDirFromRoot = "ucli";
        cargoBuildExtraArgs = features;
        additionalTestSrcFilter = path: _:
          pkgs.lib.hasPrefix "hubble/src/graphql" path;
        cargoTestExtraAttrs = {
          partitions = 1;
          partitionType = "count";
        };
      });

      ucliMainnet = (mkUcli "" "-mainnet");
      ucliMinimal = (mkUcli "--features eth-minimal" "-minimal");

    in
    {
      packages = pkgs.lib.recursiveUpdate ucliMinimal.packages ucliMainnet.packages;
    };
}
      