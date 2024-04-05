{ inputs, ... }: {
  perSystem = { pkgs, goPkgs, self', crane, system, ensureAtRepositoryRoot, dbg, ... }: {
    packages = {
      osmosisd = goPkgs.pkgsStatic.buildGoModule ({
        name = "osmosisd";
        src = inputs.osmosis;
        vendorHash = "sha256-AFWgikNPM2yHsJA113HAcrURcoNrP2IabJV8u424wRM=";
        doCheck = false;
        doInstallCheck = false;
        meta.mainProgram = "osmosisd";
        # CGO_ENABLED = 0;
        subPackages = [ "./cmd/osmosisd" ];
        buildTags = [ "netgo" ];
        GOWORK = "off";
      } // (
        let libwasmvm = self'.packages.libwasmvm;
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
        # }
        else
          { }
      ));
    };
  };
}

