{ ... }: {
  perSystem = { pkgs, self', crane, system, ensureAtRepositoryRoot, ... }:
  {
    packages = {
      wasmd = pkgs.pkgsStatic.buildGoModule ({
        name = "wasmd";
        src = pkgs.fetchFromGitHub {
          owner = "aeryz";
          repo = "wasmd";
          rev = "wasm-clients";
          sha256 = "sha256-mf1cIIPaC0cYlt/ydEu7J0L2oTPDlhX4NDD5vHT8pwg=";
        };
        vendorSha256 = null;
        doCheck = false;
        meta.mainProgram = "wasmd";
        # CGO_ENABLED = 0;
      }// (
          let libwasmvm = self'.packages.libwasmvm_1_3_0;
          in if pkgs.stdenv.isLinux then {
            # Statically link if we're on linux
            nativeBuildInputs = [ pkgs.musl libwasmvm ];
            ldflags = [
              "-linkmode external"
              "-extldflags '-z noexecstack -static -L${pkgs.musl}/lib -L${libwasmvm}/lib'"
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
        ));

        wasmd-image = pkgs.dockerTools.buildImage {
          name = "wasmd";

          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            paths = [ pkgs.coreutils-full pkgs.cacert ];
            pathsToLink = [ "/bin" ];
          };
          config = {
            Entrypoint = [ (pkgs.lib.getExe self'.packages.wasmd) ];
            Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
          };
        };
    };
  };
}
