_: {
  perSystem =
    {
      pkgs,
      buildPnpmPackage,
      self',
      ensureAtRepositoryRoot,
      ...
    }:
    let
      hash = "sha256-8fBgRR9gTZRsImkeRxVkzADjQb3DH7+KSi+MMvNdI00=";
    in
    {
      packages = {
        ts-sdk = buildPnpmPackage {
          inherit hash;
          packageJsonPath = ./package.json;
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
            cp -r ./ts-sdk/build/* $out
          '';
          checkPhase = ''
            pnpm --filter=@unionlabs/sdk check
            pnpm --filter=@unionlabs/sdk test
          '';
        };
        ts-sdk-docs = buildPnpmPackage {
          inherit hash;
          packageJsonPath = ./package.json;
          pnpmWorkspaces = [
            "@unionlabs/sdk"
          ];
          extraSrcs = [
            ../ts-sdk
          ];
          buildPhase = ''
            runHook preBuild
            pnpm --filter=@unionlabs/sdk docgen
            runHook postBuild
          '';
          installPhase = ''
            mkdir -p $out
            cp -r ./ts-sdk/docs/* $out
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
