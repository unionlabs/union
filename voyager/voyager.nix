{ ... }: {
  perSystem = { self', pkgs, system, config, crane, stdenv, ... }:
    let
      mkVoyager = features: pnameSuffix: (crane.buildWorkspaceMember {
        inherit pnameSuffix;
        crateDirFromRoot = "voyager";
        cargoBuildExtraArgs = features;
        additionalTestSrcFilter = path: _:
          (pkgs.lib.hasPrefix "hubble/src/graphql" path);
      });

      voyagerMainnet = (mkVoyager "--features eth-mainnet" "-mainnet");
      voyagerMinimal = (mkVoyager "" "-minimal");
    in
    {
      packages = pkgs.lib.recursiveUpdate voyagerMainnet.packages voyagerMinimal.packages;
      checks = pkgs.lib.recursiveUpdate voyagerMainnet.checks voyagerMinimal.checks;
    };
}
