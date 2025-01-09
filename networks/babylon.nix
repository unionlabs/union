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
        babylond = goPkgs.pkgsStatic.buildGo123Module (
          {
            name = "babylond";
            src = inputs.babylon;
            vendorHash = "sha256-wfvcbBdWBo2jYTiRymHDxL5f6X9RXr2scGuKNc5OaAY=";
            doCheck = false;
            doInstallCheck = false;
            meta.mainProgram = "babylond";
            # CGO_ENABLED = 0;
            subPackages = [ "./cmd/babylond" ];
            buildTags = [ "netgo" ];
          }
          // (
            let
              CGO_CFLAGS = "-I${self'.packages.libblst}/include -I${self'.packages.libblst.src}/src -I${self'.packages.libblst.src}/build -O";
              CGO_LDFLAGS = "-z noexecstack -static -L${goPkgs.musl}/lib -L${self'.packages.libwasmvm-2_1_3}/lib -s -w";
            in
            if pkgs.stdenv.isLinux then
              {
                inherit CGO_CFLAGS;
                inherit CGO_LDFLAGS;
                # Statically link if we're on linux
                nativeBuildInputs = [
                  pkgs.musl
                ];
                ldflags = [
                  "-linkmode external"
                ];
              }
            else
              { }
          )
        );
      };
    };
}
