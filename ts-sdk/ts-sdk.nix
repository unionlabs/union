_: {
  perSystem =
    {
      pkgs,
      pkgsUnstable,
      lib,
      self',
      ensureAtRepositoryRoot,
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
      pnpm = pkgs.pnpm_10;
    in
    {
      packages = {
        ts-sdk = buildPnpmPackage {
          inherit pnpm;
          packageJsonPath = ./package.json;
          extraSrcs = [ ../ts-sdk ];
          pnpmWorkspaces = [ "@unionlabs/sdk" ];
          hash = "sha256-z1X1EL8XLkeu3IX0LjLkoNr5FX5P24eq/QCpHxSuY5M=";
          doCheck = true;
          buildPhase = ''
            runHook preBuild
            pnpm --filter=@unionlabs/sdk build
            runHook postBuild
          '';
          installPhase = ''
            mkdir -p $out
            cp -r ./ts-sdk/* $out
          '';
          checkPhase = ''
            pnpm run --filter=@unionlabs/sdk test
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
              ${pnpm} publish --access='public'
            '';
          };
        };
        ts-sdk-fetch-schema = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "ts-sdk-fetch-schema";
            runtimeInputs = deps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd ts-sdk/
              pnpm dlx gql.tada generate-schema --tsconfig ./tsconfig.json --output "./src/generated/schema.graphql" "https://development.graphql.union.build/v1/graphql"
              pnpm dlx gql.tada generate-output --disable-preprocessing --tsconfig ./tsconfig.json --output ./src/generated/graphql-env.d.ts
            '';
          };
        };
      };
    };
}
