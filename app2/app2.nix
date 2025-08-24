_: {
  perSystem =
    {
      pkgs,
      pkgsUnstable,
      ensureAtRepositoryRoot,
      lib,
      gitShortRev,
      lastModified,
      lastModifiedDate,
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
      PUBLIC_DATADOG_CLIENT_TOKEN = "pub69b8a3deb766e91a19b44cccf0c3352e";
      PUBLIC_GIT_REV = gitShortRev;
      PUBLIC_LAST_MODIFIED_DATE = lastModifiedDate;
      PUBLIC_LAST_MODIFIED_EPOCH = lastModified;
      PUBLIC_SUPABASE_URL = "https://api.dashboard.union.build";
      PUBLIC_SUPABASE_ANON_KEY = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InVvcnF6cHVyeXJnZm5lY2FkYWpvIiwicm9sZSI6ImFub24iLCJpYXQiOjE3MzQzNzM0NDAsImV4cCI6MjA0OTk0OTQ0MH0.4xkWpfMkYgBz4nqUGkZVjQNP7NxLa4filDoJRCI3yWo";
    in
    {
      packages = {
        app2 = buildPnpmPackage rec {
          packageJsonPath = ./package.json;
          extraSrcs = [
            ../app2
            ../typescript-sdk
            ../ts-sdk
            ../ts-sdk-evm
            ../ts-sdk-cosmos
          ];
          hash = "sha256-Wc+E99V1tj+tG7VVha9CBB1YVb+H83GgUJpxhvq1gzU=";
          buildInputs = deps;
          nativeBuildInputs = buildInputs;
          pnpmWorkspaces = [
            "app2"
            "@unionlabs/sdk"
            "@unionlabs/sdk-evm"
            "@unionlabs/sdk-cosmos"
            "@unionlabs/client"
          ];
          buildPhase = ''
            runHook preBuild

            export PUBLIC_DATADOG_CLIENT_TOKEN="${PUBLIC_DATADOG_CLIENT_TOKEN}"
            export PUBLIC_GIT_REV="${PUBLIC_GIT_REV}"
            export PUBLIC_LAST_MODIFIED_DATE="${PUBLIC_LAST_MODIFIED_DATE}"
            export PUBLIC_LAST_MODIFIED_EPOCH="${PUBLIC_LAST_MODIFIED_EPOCH}"
            export PUBLIC_SUPABASE_URL="${PUBLIC_SUPABASE_URL}"
            export PUBLIC_SUPABASE_ANON_KEY="${PUBLIC_SUPABASE_ANON_KEY}"
            pnpm --filter=app2 prepare
            pnpm --filter=app2 build
            runHook postBuild
          '';
          # TODO(ehegnes): make warning diagnostics exit non-zero
          checkPhase = ''
            pnpm --filter=app2 check
          '';
          doCheck = true;
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
              export PUBLIC_DATADOG_CLIENT_TOKEN="${PUBLIC_DATADOG_CLIENT_TOKEN}"
              export PUBLIC_GIT_REV="${PUBLIC_GIT_REV}"
              export PUBLIC_LAST_MODIFIED_DATE="${PUBLIC_LAST_MODIFIED_DATE}"
              export PUBLIC_LAST_MODIFIED_EPOCH="${PUBLIC_LAST_MODIFIED_EPOCH}"
              export PUBLIC_SUPABASE_URL="${PUBLIC_SUPABASE_URL}"
              export PUBLIC_SUPABASE_ANON_KEY="${PUBLIC_SUPABASE_ANON_KEY}"
              pnpm install
              pnpm run dev --host
            '';
          };
        };
        app2-check-watch = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "app-check-watch";
            runtimeInputs = deps;
            # TODO: decrease threshold to "warning"
            text = ''
              ${ensureAtRepositoryRoot}
              cd app2/
              export PUBLIC_DATADOG_CLIENT_TOKEN="${PUBLIC_DATADOG_CLIENT_TOKEN}"
              export PUBLIC_GIT_REV="${PUBLIC_GIT_REV}"
              export PUBLIC_LAST_MODIFIED_DATE="${PUBLIC_LAST_MODIFIED_DATE}"
              export PUBLIC_LAST_MODIFIED_EPOCH="${PUBLIC_LAST_MODIFIED_EPOCH}"
              export PUBLIC_SUPABASE_URL="${PUBLIC_SUPABASE_URL}"
              export PUBLIC_SUPABASE_ANON_KEY="${PUBLIC_SUPABASE_ANON_KEY}"
              pnpm run check --watch --threshold error
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
              pnpm dlx gql.tada generate-schema --tsconfig ./tsconfig.json --output "./src/generated/schema.graphql" "https://orion.james.union.build/v1/graphql"
              pnpm dlx gql.tada generate-output --disable-preprocessing --tsconfig ./tsconfig.json --output ./src/generated/graphql-env.d.ts
            '';
          };
        };
      };
    };
}
