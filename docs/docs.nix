_: {
  perSystem =
    {
      self',
      ensureAtRepositoryRoot,
      mkCi,
      pkgsUnstable,
      buildPnpmPackage,
      ...
    }:
    let
      deps = with pkgsUnstable; [
        vips
        pkg-config
      ];
    in
    {
      packages = {
        docs = mkCi false (buildPnpmPackage {
          extraSrcs = [
            ../docs
            ../ts-sdk
            ../ts-sdk-evm
            ../ts-sdk-cosmos
            ../scripts
            ../versions
            ../deployments
          ];
          hash = "sha256-b0UEkhTc2P/GdUR85bTCN2kcfhytnFNKlJtu/TmINlo=";
          packageJsonPath = ./package.json;
          pnpmWorkspaces = [
            "docs"
            "@unionlabs/sdk"
            "@unionlabs/sdk-evm"
            "@unionlabs/sdk-cosmos"
          ];
          nativeBuildInputs = deps;
          buildInputs = deps;
          buildPhase = ''
            runHook preBuild
            export PUPPETEER_SKIP_DOWNLOAD=1
            export ASTRO_TELEMETRY_DISABLED=1
            export NODE_OPTIONS="--no-warnings"
            mkdir docs/src/content/docs/reference/@unionlabs
            cp -r ${self'.packages.ts-sdk-docs}/modules docs/src/content/docs/reference/@unionlabs/sdk
            cp -r ${self'.packages.ts-sdk-evm-docs}/modules docs/src/content/docs/reference/@unionlabs/sdk-evm
            cp -r ${self'.packages.ts-sdk-cosmos-docs}/modules docs/src/content/docs/reference/@unionlabs/sdk-cosmos
            # pnpm -w run docgen # TODO(ehegnes): enable
            pnpm --filter=docs build
            runHook postBuild
          '';
          installPhase = ''
            mkdir -p $out
            cp -r ./docs/dist/* $out
          '';
          doDist = false;
        });
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
              pnpm install
              pnpm -w run docgen
              pnpm run dev
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

              npm_config_yes=true pnpm dlx astro check
              npm_config_yes=true pnpm dlx astro build

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
              npm_config_yes=true pnpm dlx @fleek-platform/cli sites deploy
            '';
          };
        };
      };
    };
}
