{ ... }: {
  perSystem = { pkgs, lib, ensureAtRepositoryRoot, ... }:
    let
      pkgsDeps = with pkgs; [ nodejs_20 vips pkg-config ];
    in
    {
      packages = {
        site = pkgs.buildNpmPackage {
          npmDepsHash = "sha256-Yoq8eCZEj6tViTyL5Uj+sr1EOWl9AXiF3/L7DTZrfyw=";
          src = ./.;
          pname = "site";
          version = "0.0.1";
          PUPPETEER_SKIP_DOWNLOAD = true;

          # nodejs = pkgs.nodejs_20;
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
        site-dev-server = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "site-dev-server";
            runtimeInputs = pkgsDeps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd site/

              npm install
              npm run dev
            '';
          };
        };
      };
    };
}
