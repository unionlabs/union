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
        strided = pkgs.pkgsStatic.buildGo123Module (
          {
            name = "strided";
            src = inputs.stride;
            vendorHash = "sha256-rher33igRvyQ+dMhYcURnsn/RJt/Km+Z8KHzGBIJH2c=";
            doCheck = false;
            doInstallCheck = false;
            meta.mainProgram = "strided";
            subPackages = [ "./cmd/strided" ];
            # buildTags = [
            #   "netgo"
            #   "muslc"
            # ];
            tags = [
              "netgo"
              "muslc"
            ];
          }
          // (
            let
              inherit (self'.packages) libwasmvm-1_5_8;
            in
            if pkgs.stdenv.isLinux then
              {
                # Statically link if we're on linux
                nativeBuildInputs = [
                  pkgs.musl
                  libwasmvm-1_5_8
                ];
                ldflags = [
                  "-linkmode external"
                  "-extldflags '-Wl,-z,muldefs -z noexecstack -static -L${pkgs.musl}/lib -L${libwasmvm-1_5_8}/lib'"
                ];
              }
            else
              { }
          )
        );
      };
    };
}
