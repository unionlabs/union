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
      nodeDeps = with unstablePkgs; [ nodePackages_latest.nodejs ];
      combinedDeps = pkgsDeps ++ nodeDeps;
      packageJSON = lib.importJSON ./package.json;
    in
    {
      packages = {
        site = mkCi false (
          unstablePkgs.buildNpmPackage {
            npmDepsHash = "sha256-5ly/z5kRpvqphF89vHS/KU6gHNhL9Wb9KB91SWpGnS8=";
            src = ./.;
            sourceRoot = "site";
            pname = packageJSON.name;
            inherit (packageJSON) version;
            nativeBuildInputs = combinedDeps;
            buildInputs = combinedDeps;
            installPhase = ''
              mkdir -p $out
              cp -r ./.vercel/output/* $out
            '';
            doDist = false;
            PUPPETEER_SKIP_DOWNLOAD = 1;
            ASTRO_TELEMETRY_DISABLED = 1;
            NODE_OPTIONS = "--no-warnings";
          }
        );
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
