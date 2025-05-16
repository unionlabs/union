_: {
  perSystem =
    {
      pkgs,
      pkgsUnstable,
      ensureAtRepositoryRoot,
      lib,
      ...
    }:
    let
      buildPnpmPackage = import ../tools/typescript/buildPnpmPackage.nix {
        inherit pkgs lib;
      };
      deps = with pkgsUnstable; [
        python3
        stdenv.cc
        pkg-config
        nodePackages_latest.nodejs
        pnpm_10
      ];
      VITE_SUPABASE_URL = "https://api.dashboard.union.build";
      VITE_SUPABASE_ANON_KEY = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InVvcnF6cHVyeXJnZm5lY2FkYWpvIiwicm9sZSI6ImFub24iLCJpYXQiOjE3MzQzNzM0NDAsImV4cCI6MjA0OTk0OTQ0MH0.4xkWpfMkYgBz4nqUGkZVjQNP7NxLa4filDoJRCI3yWo";
    in
    {
      packages = {
        app2 = buildPnpmPackage rec {
          packageJsonPath = ./package.json;
          extraSrcs = [
            ../app2
            ../typescript-sdk
            ../ts-sdk
          ];
          hash = "sha256-a6gUnBK1NNcD5ggZxGqtNNY6a/Qz7dMjwNXbbHhIZxE=";
          buildInputs = deps;
          nativeBuildInputs = buildInputs;
          pnpmWorkspaces = [
            "app2"
            "@unionlabs/sdk"
            "@unionlabs/client"
          ];
          buildPhase = ''
            runHook preBuild
            export VITE_SUPABASE_URL="${VITE_SUPABASE_URL}"
            export VITE_SUPABASE_ANON_KEY="${VITE_SUPABASE_ANON_KEY}"
            pnpm --filter=app2 prepare
            pnpm --filter=app2 build
            runHook postBuild
          '';
          checkPhase = ''
            pnpm --filter=app2 check
          '';
          doCheck = false; # TODO(ehegnes): enable checks
          installPhase = ''
            mkdir -p $out
            cp -r ./app2/build/* $out
          '';
          doDist = false;
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
              export VITE_SUPABASE_URL="${VITE_SUPABASE_URL}"
              export VITE_SUPABASE_ANON_KEY="${VITE_SUPABASE_ANON_KEY}"
              pnpm install
              pnpm run dev -- --host
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
              pnpm dlx gql.tada generate-schema --tsconfig ./tsconfig.json --output "./src/generated/schema.graphql" "https://graphql.union.build/v1/graphql"
              pnpm dlx gql.tada generate-output --disable-preprocessing --tsconfig ./tsconfig.json --output ./src/generated/graphql-env.d.ts
            '';
          };
        };
      };
    };
}
