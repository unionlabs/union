_: {
  perSystem =
    {
      lib,
      mkCi,
      jsPkgs,
      ensureAtRepositoryRoot,
      ...
    }:
    let
      deps = with jsPkgs; [
        vips
        pkg-config
        nodePackages_latest.nodejs
      ];
      packageJSON = lib.importJSON ./package.json;
    in
    {
      packages = {
        docs = mkCi false (
          jsPkgs.buildNpmPackage {
            npmDepsHash = "sha256-tBFR+BGxNEKt3fef5pTXkCY5hGtX0f8Ntaz+hE49Dq0=";
            src = ./.;
            srcs = [
              ./.
              ./../versions/.
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
          program = jsPkgs.writeShellApplication {
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
          program = jsPkgs.writeShellApplication {
            name = "docs-check";
            runtimeInputs = deps;
            text = ''
              ${ensureAtRepositoryRoot}
              biome check docs --error-on-warnings --write --unsafe

              nix fmt

              cd docs/

              npm_config_yes=true npx astro check
              npm_config_yes=true npx astro build

              nix build .\#checks.${jsPkgs.system}.spellcheck --print-build-logs
            '';
          };
        };
        deploy-docs-ipfs = {
          type = "app";
          program = jsPkgs.writeShellApplication {
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
