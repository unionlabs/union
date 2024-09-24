{ inputs, ... }:
{
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
        starsd = goPkgs.pkgsStatic.buildGoModule (
          {
            name = "starsd";
            src = inputs.stargaze;
            vendorHash = "sha256-yg2RT1NnUaqlU8Gvlx5ZxHotSMZbBnicsAj5Jzi0BXo=";
            doCheck = false;
            doInstallCheck = false;
            meta.mainProgram = "starsd";
            # CGO_ENABLED = 0;
            subPackages = [ "./cmd/starsd" ];
            buildTags = [ "netgo" ];
          }
          // (
            let
              inherit (self'.packages) libwasmvm;
            in
            if pkgs.stdenv.isLinux then
              {
                # Statically link if we're on linux
                nativeBuildInputs = [
                  pkgs.musl
                  libwasmvm
                ];
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
          )
        );
      };
    };
}
