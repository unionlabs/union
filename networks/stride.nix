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
        strided = goPkgs.pkgsStatic.buildGoModule (
          {
            name = "strided";
            src = inputs.stride;
            vendorHash = "sha256-Tg96wuqgS08GGorY5Hbq3eJvJ7ZngI7XCqOIw84isSI=";
            doCheck = false;
            doInstallCheck = false;
            meta.mainProgram = "strided";
            subPackages = [ "./cmd/strided" ];
            buildTags = [ "netgo" ];
          }
          // (
            let
              inherit (self'.packages) libwasmvm-1_5_2;
            in
            if pkgs.stdenv.isLinux then
              {
                # Statically link if we're on linux
                nativeBuildInputs = [
                  pkgs.musl
                  libwasmvm-1_5_2
                ];
                ldflags = [
                  "-linkmode external"
                  "-extldflags '-Wl,-z,muldefs -z noexecstack -static -L${pkgs.musl}/lib -L${libwasmvm-1_5_2}/lib'"
                ];
              }
            else
              { }
          )
        );
      };
    };
}
