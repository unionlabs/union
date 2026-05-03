{ inputs, ... }:
{
  perSystem =
    {
      pkgs,
      pkgsUnstable,
      crane,
      rust,
      dbg,
      ...
    }:
    let
      gno = pkgsUnstable.pkgsStatic.buildGo124Module {
        name = "gno";
        src = inputs.gno;
        vendorHash = "sha256-8kuyN44JcnwTM0z4IdxqMdUMb7zhghfhwMx2UAW/TBw=";
        meta = {
          mainProgram = "gno";
        };
        doCheck = false;
        subPackages = [ "./gnovm/cmd/gno" ];
        ldflags = [
          "-X github.com/gnolang/gno/gnovm/pkg/gnoenv._GNOROOT=${inputs.gno}"
        ];
      };
      gnokey = pkgsUnstable.pkgsStatic.buildGo124Module {
        name = "gno";
        src = inputs.gno;
        vendorHash = "sha256-8kuyN44JcnwTM0z4IdxqMdUMb7zhghfhwMx2UAW/TBw=";
        meta = {
          mainProgram = "gnokey";
        };
        env.CGO_ENABLED = 0;
        doCheck = false;
        subPackages = [ "./gno.land/cmd/gnokey" ];
        ldflags = [
          "-X github.com/gnolang/gno/gnovm/pkg/gnoenv._GNOROOT=${inputs.gno}"
        ];
      };
      gnodev = pkgsUnstable.pkgsStatic.buildGo124Module {
        name = "gnodev";
        src = inputs.gno;
        vendorHash = "sha256-jvPVL8ih6uv/8kuVr+vwmPhO8EYC+3WaWO18RmjXAcg=";
        meta = {
          mainProgram = "gnodev";
        };
        env.CGO_ENABLED = 0;
        doCheck = false;
        prePatch = ''
          ls -alhL .
          cd ./contribs/gnodev
        '';
        ldflags = [
          "-X github.com/gnolang/gno/gnovm/pkg/gnoenv._GNOROOT=${inputs.gno}"
        ];
      };
    in
    {
      packages = { inherit gno gnodev gnokey; };
    };
}
