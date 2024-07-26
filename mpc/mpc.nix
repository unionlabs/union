{ self, ... }: {
  perSystem = { self', pkgs, crane, ... }:
    let
      attrs = {
        rustflags = "-L${self'.packages.galoisd-library}/lib";
      };
      mpc-client = crane.buildWorkspaceMember (attrs // {
        crateDirFromRoot = "mpc/client";
      });
      mpc-coordinator = crane.buildWorkspaceMember (attrs // {
        crateDirFromRoot = "mpc/coordinator";
      });
    in
    {
      packages = mpc-coordinator.packages // mpc-client.packages // {
        mpc-image = pkgs.dockerTools.buildImage {
          name = "${self'.packages.mpc-client.name}-image";
          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            paths = [ pkgs.coreutils-full pkgs.cacert ];
            pathsToLink = [ "/bin" ];
          };
          config = {
            Entrypoint = [ (pkgs.lib.getExe self'.packages.mpc) ];
            Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
          };
        };
      };
      checks = mpc-coordinator.checks // mpc-client.checks;
    };
}
