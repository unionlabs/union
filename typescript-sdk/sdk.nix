{ ... }: {
  perSystem = { pkgs, lib, ensureAtRepositoryRoot, ... }:
    let
      pkgsDeps = with pkgs; [ nodejs_20 pkg-config ];
    in
    {
      packages = {
        site = pkgs.buildNpmPackage {
          npmDepsHash = "";
          src = ./.;
          pname = "sdk";
          version = "0.0.0";
          PUPPETEER_SKIP_DOWNLOAD = true;
          nativeBuildInputs = pkgsDeps;
          buildInputs = pkgsDeps;

          installPhase = ''
            mkdir -p $out
            cp -r ./dist/* $out
          '';

          doDist = false;
        };
      };

      apps = {
      };
    };
}
