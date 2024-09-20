_: {
  perSystem =
    {
      pkgs,
      unstablePkgs,
      lib,
      ensureAtRepositoryRoot,
      mkCi,
      ...
    }:
    let
      pkgsDeps = with pkgs; [ pkg-config ];
      nodeDeps = with unstablePkgs; [
        vips
        nodePackages_latest.nodejs
      ];
      combinedDeps = pkgsDeps ++ nodeDeps;
      packageJSON = lib.importJSON ./package.json;
    in
    {
      packages = {
        docs = mkCi false (
          unstablePkgs.buildNpmPackage {
            npmDepsHash = "sha256-w9BqWfAUS+Ll1Im2plzzfQTPWLDCrKpAeJgjEhUEbH0=";
            src = ./.;
            srcs = [
              ./.
              ./../evm/.
              ./../networks/genesis/.
              ./../versions/.
            ];
            sourceRoot = "docs";
            pname = packageJSON.name;
            inherit (packageJSON) version;
            nativeBuildInputs = combinedDeps;
            buildInputs = combinedDeps;
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
          program = pkgs.writeShellApplication {
            name = "docs-dev-server";
            runtimeInputs = combinedDeps;
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
          program = pkgs.writeShellApplication {
            name = "docs-check";
            runtimeInputs = combinedDeps;
            text = ''
              ${ensureAtRepositoryRoot}
              biome check docs --error-on-warnings --write --unsafe

              nix fmt

              cd docs/

              npm_config_yes=true npx astro check
              npm_config_yes=true npx astro build

              nix build .\#checks.${pkgs.system}.spellcheck --print-build-logs
            '';
          };
        };
        deploy-docs-ipfs = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "deploy-docs-ipfs";
            runtimeInputs = combinedDeps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd docs/

              export PUPPETEER_SKIP_DOWNLOAD=1 
              nix build .#docs
              npm_config_yes=true npx @fleek-platform/cli sites deploy
            '';
          };
        };
        generate-rust-docs = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "generate-rust-docs";
            text =
              let
                /**
                 * The list of rust packages that we want to generate docs for.
                 */
                rustPackages = [ "hubble" "voyager" ];
              in
              ''
                ${ensureAtRepositoryRoot}
                
                rm -rf docs/generated/rust
                mkdir -p docs/generated/rust

                ${lib.concatMapStrings (pkg: ''
                  cargo rustdoc \
                    --release \
                    --all-features \
                    --package='${pkg}' \
                    -- \
                    --default-theme='ayu' \
                    --document-private-items

                '') rustPackages}
                cp -r target/doc/* docs/generated/rust/
              '';
          };
        };
        generate-rust-docs = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "generate-rust-docs";
            text =
              let
                /**
                 * The list of rust packages that we want to generate docs for.
                 */
                rustPackages = [ "hubble" "voyager" ];
              in
              ''
                ${ensureAtRepositoryRoot}
                
                rm -rf docs/generated/rust
                mkdir -p docs/generated/rust

                ${lib.concatMapStrings (pkg: ''
                  cargo rustdoc \
                    --release \
                    --all-features \
                    --package='${pkg}' \
                    -- \
                    --default-theme='ayu' \
                    --document-private-items

                '') rustPackages}
                cp -r target/doc/* docs/generated/rust/
              '';
          };
        };
      };
    };
}
