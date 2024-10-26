_: {
  perSystem =
    {
      lib,
      pkgs,
      jsPkgs,
      ensureAtRepositoryRoot,
      ...
    }:
    let
      deps = with jsPkgs; [
        python3
        pkg-config
        nodePackages_latest.nodejs
        nodePackages_latest."patch-package"
      ];
      packageJSON = lib.importJSON ./package.json;
    in
    {
      packages = {
        app = jsPkgs.buildNpmPackage {
          npmDepsHash = "sha256-9zyLRjVpn+2oiI56I23sQTZvWoAy3FTd5P3xnE1HQGc=";
          src = ./.;
          sourceRoot = "app";
          npmFlags = [
            "--loglevel=verbose"
            "--enable-pre-post-scripts"
          ];
          pname = packageJSON.name;
          inherit (packageJSON) version;
          nativeBuildInputs = deps;
          buildInputs = deps;
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
            runtimeInputs = deps;
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
            runtimeInputs = deps;
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
            runtimeInputs = deps;
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
