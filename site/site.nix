{ ... }: {
  perSystem = { pkgs, unstablePkgs, lib, ensureAtRepositoryRoot, mkCi, ... }:
    let
      pkgsDeps = with pkgs; [ pkg-config ];
      nodeDeps = with unstablePkgs; [ vips nodePackages_latest.nodejs ];
      combinedDeps = pkgsDeps ++ nodeDeps;
      packageJSON = lib.importJSON ./package.json;
    in
    {
      packages = {
        site = mkCi false (unstablePkgs.buildNpmPackage {
          npmDepsHash = "sha256-1o7AA7UGu16nAYraIg0qqpwx2gTyLfduGX+bKgo5d7c=";
          src = ./.;
          srcs = [ ./. ./../evm/. ./../networks/genesis/. ./../versions/. ];
          sourceRoot = "site";
          pname = packageJSON.name;
          version = packageJSON.version;
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
              
              npm_config_yes=true npx astro check
            '';
          };
        };
      };
    };
}
