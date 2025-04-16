_: {
  perSystem =
    {
      lib,
      mkCi,
      pkgsUnstable,
      ensureAtRepositoryRoot,
      ...
    }:
    let
      deps = with pkgsUnstable; [
        vips
        pkg-config
        nodePackages_latest.nodejs
      ];
      packageJSON = lib.importJSON ./package.json;
    in
    {
      packages = {
        docs = mkCi false (
          pkgsUnstable.buildNpmPackage {
            npmDepsHash = "sha256-pDdKyuOeurWp9LbYFb8/IFdvNPihtpCUi/SIvBN0ZmY=";
            src = ./.;
            srcs = [
              ./.
              ./../versions/.
              ./../deployments/.
            ];
            sourceRoot = "docs";
            pname = packageJSON.name;
            inherit (packageJSON) version;
            nativeBuildInputs = deps;
            buildInputs = deps;
            installPhase = ''
              mkdir -p $out
              cp -r ./dist/* $out
            '';
            doDist = false;
            PUPPETEER_SKIP_DOWNLOAD = 1;
            ASTRO_TELEMETRY_DISABLED = 1;
            NODE_OPTIONS = "--no-warnings";
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
