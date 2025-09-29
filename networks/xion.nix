{ inputs, ... }:
{
  perSystem =
    {
      pkgs,
      pkgsUnstable,
      self',
      crane,
      system,
      ensureAtRepositoryRoot,
      dbg,
      ...
    }:
    {
      packages = {
        xiond = pkgsUnstable.pkgsStatic.buildGoModule (
          {
            name = "xiond";
            src = inputs.xion;
            vendorHash = "sha256-LqqtiZCAVLush/0aHQbrICHR8LvYtQ1MU9ba3oPzv8A=";
            doCheck = false;
            doInstallCheck = false;
            meta.mainProgram = "xiond";
            # CGO_ENABLED = 0;
            subPackages = [ "./cmd/xiond" ];
            tags = [
              "netgo"
              "muslc"
            ];
            env.GOWORK = "off";
          }
          // (
            let
              inherit (self'.packages) libwasmvm-3_0_0;
            in
            if pkgs.stdenv.isLinux then
              {
                # Statically link if we're on linux
                nativeBuildInputs = [
                  pkgs.musl
                  libwasmvm-3_0_0
                ];
                ldflags = [
                  "-linkmode external"
                  "-extldflags '-Wl,-z,muldefs -z noexecstack -static -L${pkgs.musl}/lib -L${libwasmvm-3_0_0}/lib'"
                ];
              }
            else
              { }
          )
        );
      };
    };
}
