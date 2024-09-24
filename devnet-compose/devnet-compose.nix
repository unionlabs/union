_: {
  perSystem =
    {
      self',
      pkgs,
      crane,
      ...
    }:
    let
      devnet-compose = crane.buildWorkspaceMember {
        crateDirFromRoot = "devnet-compose";
      };
    in
    {
      packages = {
        inherit (devnet-compose.packages) devnet-compose;

        devnet-compose-image = pkgs.dockerTools.buildLayeredImage {
          name = "devnet-compose";
          contents = [
            pkgs.coreutils-full
            pkgs.cacert
            self'.packages.devnet-compose
          ];
          config = {
            Entrypoint = [ (pkgs.lib.getExe self'.packages.devnet-compose) ];
            Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
          };
        };
      };
    };

}
