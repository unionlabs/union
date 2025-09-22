_: {
  perSystem =
    {
      pkgs,
      buildPnpmPackage,
      self',
      ensureAtRepositoryRoot,
      ...
    }:
    {
      packages = {
        ts-sdk = buildPnpmPackage {
          packageJsonPath = ./package.json;
          hash = "sha256-X+yOSBK99AnS11sXHfQuQjqgjkxrCJms4z+A+Xrh8Ss=";
          pnpmWorkspaces = [
            "@unionlabs/sdk"
          ];
          extraSrcs = [
            ../ts-sdk
          ];
          doCheck = true;
          buildPhase = ''
            runHook preBuild
            pnpm --filter=@unionlabs/sdk build
            runHook postBuild
          '';
          installPhase = ''
            mkdir -p $out
            echo TS SDK NODE ROOT
            ls -lah ./ts-sdk
            echo TS SDK NODE NODE MODULES
            ls -lah ./ts-sdk/node_modules
            cp -r ./ts-sdk/build/* $out
          '';
          checkPhase = ''
            pnpm --filter=@unionlabs/sdk check
            pnpm --filter=@unionlabs/sdk test
          '';
        };
      };
      apps = {
        publish-ts-sdk = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "publish-ts-sdk";
            text = ''
              cd ${self'.packages.ts-sdk}/
              pnpm publish --access='public'
            '';
          };
        };
        ts-sdk-fetch-schema = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "ts-sdk-fetch-schema";
            text = ''
              ${ensureAtRepositoryRoot}
              cd ts-sdk/
              pnpm dlx gql.tada generate-schema --tsconfig ./tsconfig.json --output "./src/generated/schema.graphql" "https://graphql.union.build/v1/graphql"
              pnpm dlx gql.tada generate-output --disable-preprocessing --tsconfig ./tsconfig.json --output ./src/generated/graphql-env.d.ts
            '';
          };
        };
      };
    };
}
