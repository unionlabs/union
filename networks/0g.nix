{ inputs, ... }: {
  perSystem = { pkgs, goPkgs, self', crane, system, ensureAtRepositoryRoot, dbg, ... }: {
    packages = {
      Ogchaind = goPkgs.pkgsStatic.buildGoModule ({
        name = "0gchaind";
        src = inputs.Ogchain;
        vendorHash = "sha256-URF/nYpUE4En0ofBi9OR5c3trqtgqWVDanhS7CYEL9w=";
        doCheck = false;
        doInstallCheck = false;
        meta.mainProgram = "0gchaind";
        # CGO_ENABLED = 0;
        subPackages = [ "./cmd/0gchaind" ];
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

