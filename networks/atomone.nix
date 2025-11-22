{ inputs, ... }:
{
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
        atomoned = pkgs.pkgsStatic.buildGo123Module (
          {
            name = "atomoned";
            src = inputs.atomone;
            vendorHash = "sha256-m1pH6cCWu88P3fSLzEQxkgCtGVzizyNDcJBu7QnoBYU=";
            doCheck = false;
            doInstallCheck = false;
            meta.mainProgram = "atomoned";
            # CGO_ENABLED = 0;
            subPackages = [ "./cmd/atomoned" ];
            tags = [
              "netgo"
              # "muslc"
            ];
            CGO_ENABLED = 0;
            env.GOWORK = "off";
          }
          // (
            let
              inherit (self'.packages) libwasmvm-2_1_3;
            in
            if pkgs.stdenv.isLinux then
              {
                # Statically link if we're on linux
                # nativeBuildInputs = [
                #   pkgs.musl
                #   libwasmvm-2_1_3
                # ];
                # ldflags = [
                #   "-linkmode external"
                #   "-extldflags '-Wl,-z,muldefs -z noexecstack -static -L${pkgs.musl}/lib -L${libwasmvm-2_1_3}/lib'"
                # ];
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
          )
        );
      };
    };
}
