_: {
  perSystem =
    {
      pkgs,
      goPkgs,
      self',
      crane,
      system,
      ensureAtRepositoryRoot,
      dbg,
      ...
    }:
    {
      packages = {
        simd = goPkgs.pkgsStatic.buildGo123Module (
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
          }
          // (
            let
              libwasmvm = self'.packages.libwasmvm-2_1_2;
            in
            # libwasmvm = pkgs.stdenv.mkDerivation {
            #   name = "libwasmvm";
            #   src = pkgs.fetchurl {
            #     url = "https://github.com/CosmWasm/wasmvm/releases/download/v2.2.0-rc.2/libwasmvm_muslc.x86_64.a";
            #     hash = "sha256-LEl7UkbHIXpwxEfFARfH+wmQnsI+bkFRpN4+XynbgTQ=";
            #   };
            #   dontUnpack = true;
            #   buildPhase = ''
            #     mkdir -p $out/lib/
            #     cp $src $out/lib/libwasmvm.x86_64.a
            #   '';
            # };
            if pkgs.stdenv.isLinux then
              {
                # Statically link if we're on linux
                nativeBuildInputs = [
                  goPkgs.musl
                  libwasmvm
                ];
                ldflags = [
                  "-linkmode external"
                  "-extldflags '-z noexecstack -static -L${goPkgs.musl}/lib -L${libwasmvm}/lib'"
                ];
              }
            # else if pkgs.stdenv.isDarwin then {
            #   # Dynamically link if we're on darwin by wrapping the program
            #   # such that the DYLD_LIBRARY_PATH includes libwasmvm
            #   buildInputs = [ pkgs.makeWrapper libwasmvm ];
            #   postFixup = ''
            #     wrapProgram $out/bin/wasmd \
            #     --set DYLD_LIBRARY_PATH ${(pkgs.lib.makeLibraryPath [ libwasmvm ])};
            #   '';
            # } else
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
