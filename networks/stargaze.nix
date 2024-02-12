{ inputs, ... }: {
  perSystem = { pkgs, goPkgs, self', crane, system, ensureAtRepositoryRoot, ... }:
    {
      packages = {
        stargaze = goPkgs.pkgsStatic.buildGoModule ({
          name = "stargaze";
          src = inputs.stargaze;
          vendorHash = "sha256-0icwBUr/jAmYk0/8lVE0THrEExuXBTJkv49/1IWQ33Y=";
          doCheck = false;
          doInstallCheck = false;
          meta.mainProgram = "starsd";
          # CGO_ENABLED = 0;
          subPackages = [ "./cmd/starsd" ];
          buildTags = [ "netgo" ];
        } // (
          let libwasmvm = self'.packages.libwasmvm-1_5_0;
          in if pkgs.stdenv.isLinux then {
            # Statically link if we're on linux
            nativeBuildInputs = [ pkgs.musl libwasmvm ];
            ldflags = [
              "-linkmode external"
              "-extldflags '-Wl,-z,muldefs -z noexecstack -static -L${pkgs.musl}/lib -L${libwasmvm}/lib'"
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
      };
    };
}

