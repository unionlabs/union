{ ... }: {
  perSystem = { self', pkgs, crane, ... }:
    let
      hubble = crane.buildWorkspaceMember {
        crateDirFromRoot = "hubble";
        additionalSrcFilter = path: _type: pkgs.lib.hasPrefix "hubble/src/graphql/" path;
        cargoTestExtraAttrs = {
          partitions = 1;
          partitionType = "count";
        };
      };
    in
    {
      inherit (hubble) checks ;
      packages = {
        hubble = hubble.packages.hubble;

        hubble-image = pkgs.dockerTools.buildImage {
          name = "hubble";

          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            paths = [ pkgs.coreutils-full  hubble ];
            pathsToLink = [ "/bin" ];
          };
          config = {
            Entrypoint = [ (pkgs.lib.getExe self'.packages.uniond) ];
          };
        };
      };
    };
}
