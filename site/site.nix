{ ... }: {
  perSystem = { pkgs, unstablePkgs, lib, ensureAtRepositoryRoot, ... }:
    let
      pkgsDeps = with pkgs; [ pkg-config ];
      nodeDeps = with unstablePkgs; [ vips nodejs_21 ];
      combinedDeps = pkgsDeps ++ nodeDeps;
    in
    {
      packages = {
        site = unstablePkgs.buildNpmPackage {
<<<<<<< HEAD
          npmDepsHash = "sha256-dd0rU/5WHM0NVxJWjGLM34VS3Y4Hmc4vRIot49E+NuE=";
=======
          npmDepsHash = "sha256-CxLgfc7+BpRKgD88L9Q8+OGWWbft8GY0DDG1Cbl1T8M=";
>>>>>>> a42b705f6 (chore: update nixpkgs-unstable input instead)
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
        };
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
      };
    };
}
