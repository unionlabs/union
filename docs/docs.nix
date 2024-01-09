{ ... }: {
  perSystem = { pkgs, lib, ensureAtRepositoryRoot, ... }:
    let
      pkgsDeps = with pkgs; [ nodejs_20 vips pkg-config ];
    in
    {
      packages = {
        docs = pkgs.buildNpmPackage {
          npmDepsHash = "sha256-J0xajg6qOZDvC3tOzj7uipDtNpRsc+s4W8aXJjM9TyE=";
          src = ./.;
          pname = "docs";
          version = "0.0.1";
          PUPPETEER_SKIP_DOWNLOAD = true;

          nodejs = pkgs.nodejs_20;
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
        docs-dev-server = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "docs-dev-server";
            runtimeInputs = pkgsDeps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd docs/

              npm install
              npm run dev
            '';
          };
        };
      };
    };
}
