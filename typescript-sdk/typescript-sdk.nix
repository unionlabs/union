_: {
  perSystem =
    {
      pkgs,
      unstablePkgs,
      ensureAtRepositoryRoot,
      ...
    }:
    {
      apps = {
        ts-sdk-publish = {
          type = "app";
          program = unstablePkgs.writeShellApplication {
            name = "ts-sdk-publish";
            text = ''
              ${ensureAtRepositoryRoot}

              cd typescript-sdk
              bun run build
              npm publish --access='public' --no-git-tagsh
            '';
          };
        };
        ts-sdk-fetch-schema = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "ts-sdk-fetch-schema";
            text = ''
              ${ensureAtRepositoryRoot}
              cd typescript-sdk/
              npx gql.tada generate-schema --tsconfig ./tsconfig.json --output "./src/generated/schema.graphql" "https://staging.graphql.union.build/v1/graphql"

              npx gql.tada generate-output --disable-preprocessing --tsconfig ./tsconfig.json --output ./src/generated/graphql-env.d.ts
            '';
          };
        };
        ts-sdk-check = {
          type = "app";
          program = unstablePkgs.writeShellApplication {
            name = "ts-sdk-check";
            text = ''
              ${ensureAtRepositoryRoot}
              biome check typescript-sdk --error-on-warnings --write --unsafe

              cd typescript-sdk
              bun run typecheck

              nix fmt

              nix build .\#checks.${pkgs.system}.spellcheck --print-build-logs
            '';
          };
        };
      };
    };
}
