_: {
  perSystem =
    {
      pkgs,
      self',
      crane,
      system,
      ensureAtRepositoryRoot,
      dbg,
      ...
    }:
    {
      packages = {
        simd = pkgs.pkgsStatic.buildGo123Module (
          {
            name = "simd";
            src = pkgs.fetchFromGitHub {
              owner = "cosmwasm";
              repo = "wasmd";
              rev = "de7db0dc672e7beb201e06e7eb12b2de356ac7c9";
              sha256 = "sha256-X8Q93gqk+gBJwn4EIxFVeWqRpHcIxNAplfARejHwfbk=";
            };
            vendorHash = "sha256-rhuYWhaTtrHCeO9l4uiP7L2OmWkCPtMHXBqS7TRzM4s=";
            subPackages = [ "./cmd/wasmd" ];
            doCheck = false;
            doInstallCheck = false;
            meta.mainProgram = "wasmd";
            tags = [
              "netgo"
              "muslc"
            ];
          }
          // (
            let
              libwasmvm = self'.packages.libwasmvm-2_1_2;
            in
            if pkgs.stdenv.isLinux then
              {
                # Statically link if we're on linux
                nativeBuildInputs = [
                  pkgs.musl
                  libwasmvm
                ];
                ldflags = [
                  "-extldflags '-z noexecstack -static -L${pkgs.musl}/lib -L${libwasmvm}/lib'"
                ];
              }
            else
              { }
          )
        );

        simd-image = pkgs.dockerTools.buildImage {
          name = "simd";

          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            paths = [
              pkgs.coreutils-full
              pkgs.cacert
            ];
            pathsToLink = [ "/bin" ];
          };
          config = {
            Entrypoint = [ (pkgs.lib.getExe self'.packages.simd) ];
            Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
          };
        };
      };
    };
}
