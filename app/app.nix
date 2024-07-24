{ ... }: {
  perSystem = { pkgs, unstablePkgs, lib, ensureAtRepositoryRoot, ... }:
    let
      pkgsDeps = with pkgs; [ pkg-config python3 ];
      nodeDeps = with unstablePkgs; [ nodePackages_latest.nodejs ];
      combinedDeps = pkgsDeps ++ nodeDeps;
      packageJSON = lib.importJSON ./package.json;
    in
    {
      packages = {
        app = unstablePkgs.buildNpmPackage {
          npmDepsHash = "sha256-PGRqqSoDeofuxWD8d7xwYYIOoiKj5Y8YlpQgdkuxs/0=";
          src = ./.;
          sourceRoot = "app";
          npmFlags = [ "--legacy-peer-deps" ];
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
              node_modules/.bin/gql.tada generate-schema \
                --tsconfig ./tsconfig.json \
                --output "./src/generated/schema.graphql" \
                "https://noble-pika-27.hasura.app/v1/graphql"

              node_modules/.bin/gql.tada generate-output \
                --disable-preprocessing \
                --tsconfig ./tsconfig.json \
                --output ./src/generated/graphql-env.d.ts
            '';
          };
        };

        app-svelte-check = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "app-svelte-check";
            runtimeInputs = combinedDeps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd app/
              
              node_modules/.bin/svelte-check sync
              node_modules/.bin/svelte-check --tsconfig ./tsconfig.json
            '';
          };
        };
      };
    };
}
