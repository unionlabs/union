_: {
  perSystem =
    {
      pkgs,
      lib,
      mkCi,
      pkgsUnstable,
      ensureAtRepositoryRoot,
      ...
    }:
    let
      deps = with pkgsUnstable; [
        pkg-config
        nodePackages_latest.nodejs
      ];
      buildPnpmPackage = import ../tools/typescript/buildPnpmPackage.nix {
        inherit lib pkgs;
      };
    in
    {
      packages = {
        site = mkCi false (
          buildPnpmPackage {
            hash = "sha256-e6FKW8iLRPfVs0httjJj9mu5UiSTASbFjai5VgMwWPY=";
            packageJsonPath = ./package.json;
            extraSrcs = [
              ../site
            ];
            nativeBuildInputs = deps;
            buildInputs = deps;
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
          }
        );
      };

      apps = {
        site-dev-server = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "site-dev-server";
            runtimeInputs = deps;
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
            runtimeInputs = deps;
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
