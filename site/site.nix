_: {
  perSystem =
    {
      pkgs,
      mkCi,
      ensureAtRepositoryRoot,
      buildPnpmPackage,
      ...
    }:
    {
      packages = {
        site = mkCi false (buildPnpmPackage {
          hash = "sha256-QQtlK9xlLoxkTgpEoHFExYcj+/Y+KjU1Uy4tVbnXdeI=";
          packageJsonPath = ./package.json;
          extraSrcs = [
            ../site
          ];
          pnpmWorkspaces = [
            "site"
          ];
          buildPhase = ''
            runHook preBuild
            export PUPPETEER_SKIP_DOWNLOAD=1;
            export ASTRO_TELEMETRY_DISABLED=1;
            export NODE_OPTIONS="--no-warnings";
            pnpm --filter=site build
            runHook postBuild
          '';
          installPhase = ''
            mkdir -p $out
            cp -r ./site/.vercel/output/* $out
          '';
          doDist = false;
        });
      };

      apps = {
        site-dev-server = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "site-dev-server";
            text = ''
              ${ensureAtRepositoryRoot}
              cd site/

              export PUPPETEER_SKIP_DOWNLOAD=1 
              npm install
              npm run dev -- --host
            '';
          };
        };
        site-check = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "site-check";
            text = ''
              ${ensureAtRepositoryRoot}
              cd site/
              npm_config_yes=true npx astro check
            '';
          };
        };
      };
    };
}
