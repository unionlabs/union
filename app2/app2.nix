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
        stdenv.cc
        pkg-config
        nodePackages_latest.nodejs
        nodePackages_latest."patch-package"
      ];
      packageJSON = lib.importJSON ./package.json;
    in
    {
      packages = {
        app2 = jsPkgs.buildNpmPackage {
          npmDepsHash = "sha256-R+7qoYHPuUmBKnvBwjsb4arZQikBckKndQlSYH2lQ5I=";
          src = ./.;
          sourceRoot = "app2";
          npmFlags = [ "--legacy-peer-deps" ];
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
        app2-dev-server = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "app-dev-server";
            runtimeInputs = deps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd app2/

              npm install
              npm run dev -- --host
            '';
          };
        };
        app2-fetch-schema = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "app2-fetch-schema";
            runtimeInputs = deps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd app2/
              npx gql.tada generate-schema --tsconfig ./tsconfig.json --output "./src/generated/schema.graphql" "https://development.graphql.union.build/v1/graphql"

              npx gql.tada generate-output --disable-preprocessing --tsconfig ./tsconfig.json --output ./src/generated/graphql-env.d.ts
            '';
          };
        };
      };
    };
}
