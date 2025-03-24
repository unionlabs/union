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
            vendorHash = "sha256-FdMcbZVodd83gVPPPXLOsN9Iq8EP7SPxshKe4sX7IDg=";
            doCheck = false;
            doInstallCheck = false;
            meta.mainProgram = "babylond";
            # CGO_ENABLED = 0;
            subPackages = [ "./cmd/babylond" ];
            tags = [
              "netgo"
              "muslc"
            ];
          }
          // (
            let
              CGO_CFLAGS = "-I${self'.packages.libblst}/include -I${self'.packages.libblst.src}/src -I${self'.packages.libblst.src}/build -O";
              CGO_LDFLAGS = "-z noexecstack -static -L${goPkgs.musl}/lib -L${self'.packages.libwasmvm-2_2_1}/lib -s -w";
            in
            if pkgs.stdenv.isLinux then
              {
                inherit CGO_CFLAGS;
                inherit CGO_LDFLAGS;
                # Statically link if we're on linux
                nativeBuildInputs = [
                  goPkgs.musl
                ];
                ldflags = [
                  "-linkmode external"
                  "-extldflags \"-Wl,-z,muldefs -static\""
                ];
              }
            else
              { }
          )
        );
      };
    };
}
