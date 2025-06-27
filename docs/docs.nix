_: {
  perSystem =
    {
      ensureAtRepositoryRoot,
      lib,
      mkCi,
      pkgs,
      pkgsUnstable,
      ...
    }:
    let
      buildPnpmPackage = import ../tools/typescript/buildPnpmPackage.nix {
        inherit pkgs lib;
      };
      deps = with pkgsUnstable; [
        vips
        pkg-config
        nodePackages_latest.nodejs
        pnpm_10
      ];
      pnpm = pkgs.pnpm_10;
    in
    {
      packages = {
        docs = mkCi false (
          buildPnpmPackage {
            inherit pnpm;
            extraSrcs = [
              ../docs
              ../ts-sdk
              ../scripts
              ../versions
              ../deployments
            ];
            hash = "sha256-H6c1DM2jPZPAvAMVqsQi3Twnz1znDA7lz7G+bKwPMV8=";
            packageJsonPath = ./package.json;
            pnpmWorkspaces = [
              "docs"
              "@unionlabs/sdk"
            ];
            nativeBuildInputs = deps;
            buildInputs = deps;
            buildPhase = ''
              runHook preBuild
              export PUPPETEER_SKIP_DOWNLOAD=1
              export ASTRO_TELEMETRY_DISABLED=1
              export NODE_OPTIONS="--no-warnings"
              pnpm run docgen
              pnpm --filter=docs build
              runHook postBuild
            '';
            installPhase = ''
              mkdir -p $out
              cp -r ./docs/dist/* $out
            '';
            doDist = false;
          }
        );
      };

      apps = {
        docs-dev-server = {
          type = "app";
          program = pkgsUnstable.writeShellApplication {
            name = "docs-dev-server";
            runtimeInputs = deps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd docs/

              export PUPPETEER_SKIP_DOWNLOAD=1 
              npm install
              npm run dev
            '';
          };
        };
        docs-check = {
          type = "app";
          program = pkgsUnstable.writeShellApplication {
            name = "docs-check";
            runtimeInputs = deps;
            text = ''
              ${ensureAtRepositoryRoot}
              biome check docs --error-on-warnings --write --unsafe

              nix fmt

              cd docs/

              npm_config_yes=true npx astro check
              npm_config_yes=true npx astro build

              nix build .\#checks.${pkgsUnstable.system}.spellcheck --print-build-logs
            '';
          };
        };
        deploy-docs-ipfs = {
          type = "app";
          program = pkgsUnstable.writeShellApplication {
            name = "deploy-docs-ipfs";
            runtimeInputs = deps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd docs/

              export PUPPETEER_SKIP_DOWNLOAD=1 
              nix build .#docs
              npm_config_yes=true npx @fleek-platform/cli sites deploy
            '';
          };
        };
      };
    };
}
