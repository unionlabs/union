{ ... }: {
  perSystem = { pkgs, lib, ensureAtRepositoryRoot, ... }:
    let pkgsDeps = with pkgs; [ nodejs_20 pkg-config ];
    in {
      packages = {
        app = pkgs.buildNpmPackage {
          npmDepsHash = "sha256-E4u/FQ++6KroGrXRIM8p38390k0EDBkSLGhxMNRN9jU=";
          src = ./.;
          sourceRoot = "app";
          pname = "app";
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
        app-dev-server = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "app-dev-server";
            runtimeInputs = pkgsDeps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd app/

              npm install
              npm run dev
            '';
          };
        };
      };
    };
}
