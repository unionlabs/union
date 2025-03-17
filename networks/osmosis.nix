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
        osmosisd = goPkgs.pkgsStatic.buildGo123Module (
          {
            name = "osmosisd";
            src = inputs.osmosis;
            vendorHash = "sha256-Sgggqfem3a5KuFP9Z05a8Xtpgl00lNBVc8+encmRHs4=";
            doCheck = false;
            doInstallCheck = false;
            meta.mainProgram = "osmosisd";
            # CGO_ENABLED = 0;
            subPackages = [ "./cmd/osmosisd" ];
            tags = [
              "netgo"
              "muslc"
            ];
            env.GOWORK = "off";
          }
          // (
            let
              inherit (self'.packages) libwasmvm-2_1_3;
            in
            if pkgs.stdenv.isLinux then
              {
                # Statically link if we're on linux
                nativeBuildInputs = [
                  pkgs.musl
                  libwasmvm-2_1_3
                ];
                ldflags = [
                  "-linkmode external"
                  "-extldflags '-Wl,-z,muldefs -z noexecstack -static -L${pkgs.musl}/lib -L${libwasmvm-2_1_3}/lib'"
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
          )
        );
      };
    };
}
