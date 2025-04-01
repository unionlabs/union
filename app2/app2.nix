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
      buildPnpmPackage = import ../nix/buildPnpmPackage.nix {
        inherit pkgs lib;
      };
      deps = with pkgsUnstable; [
        python3
        stdenv.cc
        pkg-config
        nodePackages_latest.nodejs
        pnpm_10
      ];
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
          hash = "sha256-nR7H08cOxhjn1PgbvdBdDoLahV+xQCFw0Xpoi2oCfII=";
          buildInputs = deps;
          nativeBuildInputs = buildInputs;
          pnpmWorkspaces = [
            "app2"
            "@unionlabs/sdk"
            "@unionlabs/client"
          ];
          buildPhase = ''
            runHook preBuild
            pnpm --filter=app2 prepare
            pnpm --filter=app2 build
            runHook postBuild
          '';
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
              pnpm dlx gql.tada generate-schema --tsconfig ./tsconfig.json --output "./src/generated/schema.graphql" "https://development.graphql.union.build/v1/graphql"
              pnpm dlx gql.tada generate-output --disable-preprocessing --tsconfig ./tsconfig.json --output ./src/generated/graphql-env.d.ts
            '';
          };
        };
      };
    };
}
