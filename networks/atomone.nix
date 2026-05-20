{ inputs, ... }:
{
  perSystem =
    {
      pkgs,
      pkgsGoUnstable,
      self',
      crane,
      system,
      ensureAtRepositoryRoot,
      dbg,
      ...
    }:
    {
      packages = {
        atomoned = pkgsGoUnstable.pkgsStatic.buildGo126Module (
          {
            name = "atomoned";
            src = inputs.atomone;
            vendorHash = "sha256-twJxXNZWMOQLS+81RWddxou4zn3Lk+WEWMXoCysln54=";
            doCheck = false;
            doInstallCheck = false;
            meta.mainProgram = "atomoned";
            # CGO_ENABLED = 0;
            subPackages = [ "./cmd/atomoned" ];
            tags = [
              "netgo"
              # "muslc"
            ];
            env = {
              GOWORK = "off";
              CGO_ENABLED = 0;
            };
          }
          // (
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
