{ ... }: {
  perSystem = { pkgs, unstablePkgs, lib, ensureAtRepositoryRoot, mkCi, ... }:
    let
      pkgsDeps = with pkgs; [ pkg-config ];
      nodeDeps = with unstablePkgs; [ vips nodePackages_latest.nodejs ];
      combinedDeps = pkgsDeps ++ nodeDeps;
    in
    {
      packages = {
<<<<<<< HEAD
        site = mkCi false (unstablePkgs.buildNpmPackage {
          npmDepsHash = "sha256-iacabyQyragOOO1AsWc0+N14e9cyl2p3Kq+egRaSGYc=";
=======
        site = unstablePkgs.buildNpmPackage {
          npmDepsHash = "sha256-IQVZ2SaabqwerS9zIwB7qLhg2JUWy5+xdXYn8uFVNCg=";
>>>>>>> 2fddec7ce (chore: update deps)
          src = ./.;
          srcs = [ ./. ./../evm/. ./../networks/genesis/. ./../versions/. ];
          sourceRoot = "site";
          pname = "site";
          version = "0.0.1";
          nativeBuildInputs = combinedDeps;
          buildInputs = combinedDeps;
          installPhase = ''
            mkdir -p $out
            cp -r ./dist/* $out
          '';
          doDist = false;
          PUPPETEER_SKIP_DOWNLOAD = 1;
          NODE_OPTIONS = "--no-warnings";
          ASTRO_TELEMETRY_DISABLED = 1;
        });
      };

      apps = {
        site-dev-server = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "site-dev-server";
            runtimeInputs = combinedDeps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd site/

              export PUPPETEER_SKIP_DOWNLOAD=1 
              npm install
              npm run dev
            '';
          };
        };

        site-check = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "site-check";
            runtimeInputs = combinedDeps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd site/
              npm run check
            '';
          };
        };
      };
    };
}
