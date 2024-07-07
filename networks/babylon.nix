{ inputs, ... }: {
  perSystem = { pkgs, goPkgs, self', crane, system, ensureAtRepositoryRoot, dbg, ... }: 
  let
      libwasmvm = self'.packages.libwasmvm-2_0_1;
      CGO_CFLAGS = "-I${self'.packages.libblst}/include -I${self'.packages.libblst.src}/src -I${self'.packages.libblst.src}/build -I${self'.packages.bls-eth.src}/bls/include -O";
      CGO_LDFLAGS = "-z noexecstack -static -L${pkgs.musl}/lib -L${libwasmvm}/lib -L${self'.packages.bls-eth}/lib -s -w";
      # CGO_LD_TEST_FLAGS = "-L${self'.packages.bls-eth}/lib";
  in
  {
    packages = {
      babylond = goPkgs.pkgsStatic.buildGoModule ({
        name = "babylond";
        src = inputs.babylon;
        vendorHash = null;
        doCheck = false;
        doInstallCheck = false;
        meta.mainProgram = "babylond";
        # CGO_ENABLED = 0;
        subPackages = [ "./cmd/babylond" ];
        buildTags = [ "netgo" ];
        GOWORK = "off";
      } // (
        if pkgs.stdenv.isLinux then {
          inherit CGO_CFLAGS;
          inherit CGO_LDFLAGS;
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

