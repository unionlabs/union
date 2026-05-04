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
      gnopls = pkgsUnstable.pkgsStatic.buildGo125Module rec {
        name = "gnopls";
        src = pkgs.fetchFromGitHub {
          owner = "gnoverse";
          repo = "gnopls";
          rev = "32e82ac207a551ee04fce8559b96e70daca083f9";
          hash = "sha256-0E2Q93537xx6tVm/8oLUYgxKQ4+1NrEnTmMz8uK+xW4=";
        };
        subPackages = [ "." ];
        vendorHash = "sha256-BD5lx+iTrj4GInH1gIyjj6B+DLPv3VGs5OpnvM0jFok=";
        ldflags = [
          "-X github.com/gnolang/gnoverse/gnopls/pkg/gnotypes._GNOBUILTIN=${src}/pkg/gnotypes/builtin"
        ];
      };
    in
    {
      packages = {
        inherit
          gno
          gnodev
          gnokey
          gnopls
          ;
      };
    };
}
