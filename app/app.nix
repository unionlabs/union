{ ... }: {
  perSystem = { pkgs, unstablePkgs, lib, ensureAtRepositoryRoot, ... }:
    let
      pkgsDeps = with pkgs; [ pkg-config python3 ];
      nodeDeps = with unstablePkgs; [ nodePackages_latest.nodejs nodePackages_latest."patch-package" ];
      combinedDeps = pkgsDeps ++ nodeDeps;
      packageJSON = lib.importJSON ./package.json;
    in
    {
      packages = {
        app = unstablePkgs.buildNpmPackage {
          npmDepsHash = "sha256-KIxRD0SCb5yex5HmoZ3BbmNzyt7Y5RuZKdwuc3gtX78=";
          src = ./.;
          sourceRoot = "app";
          npmFlags = [ "--enable-pre-post-scripts" ];
          pname = packageJSON.name;
          version = packageJSON.version;
          nativeBuildInputs = combinedDeps;
          buildInputs = combinedDeps;
          installPhase = ''
            mkdir -p $out
            cp -r ./build/* $out
          '';
          doDist = false;
          NODE_OPTIONS = "--no-warnings";
        };
      };

      apps = {
        app-dev-server = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "app-dev-server";
            runtimeInputs = combinedDeps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd app/

              npm install
              npm run dev
            '';
          };
        };
        app-fetch-schema = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "app-dev-server";
            runtimeInputs = combinedDeps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd app/
              npx gql.tada generate-schema --tsconfig ./tsconfig.json --output "./src/generated/schema.graphql" "https://blue.graphql.union.build/v1/graphql"

              npx gql.tada generate-output --disable-preprocessing --tsconfig ./tsconfig.json --output ./src/generated/graphql-env.d.ts
            '';
          };
        };
        deploy-app-ipfs = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "deploy-app-ipfs";
            runtimeInputs = combinedDeps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd app/

              nix build .#app
              npm_config_yes=true npx @fleek-platform/cli sites deploy
            '';
          };
        };
      };
    };
}
