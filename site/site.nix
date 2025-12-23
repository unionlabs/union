_: {
  perSystem =
    {
      pkgs,
      mkCi,
      ensureAtRepositoryRoot,
      buildPnpmPackage,
      ...
    }:
    let
      deps = with pkgs; [
        python3
        stdenv.cc
        pkg-config
      ];
    in
    {
      packages = {
        site = mkCi false (buildPnpmPackage {
          hash = "sha256-WkDdok2luYp/q1s2cb4Aa7iWzKsifJ7HpRKMnf0pTG4=";
          packageJsonPath = ./package.json;
          extraSrcs = [
            ../site
          ];
          pnpmWorkspaces = [
            "site"
          ];
          buildInputs = deps;
          nativeBuildInputs = deps;
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
            runtimeInputs = deps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd site/

              export PUPPETEER_SKIP_DOWNLOAD=1 
              pnpm install
              pnpm dev -- --host
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
