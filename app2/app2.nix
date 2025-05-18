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
          hash = "sha256-wydMzX2aX/1QqAwtQlWWo5+8O65NprPFLxbYDMHgE3E=";
          buildInputs = deps;
          nativeBuildInputs = buildInputs;
          pnpmWorkspaces = [
            "app2"
            "@unionlabs/sdk"
            "@unionlabs/client"
          ];
          buildPhase = ''
            runHook preBuild
            export PUBLIC_DATADOG_CLIENT_TOKEN="${PUBLIC_DATADOG_CLIENT_TOKEN}"
            export PUBLIC_GIT_REV="${PUBLIC_GIT_REV}"
            export PUBLIC_LAST_MODIFIED_DATE="${PUBLIC_LAST_MODIFIED_DATE}"
            export PUBLIC_LAST_MODIFIED_EPOCH="${PUBLIC_LAST_MODIFIED_EPOCH}"
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
              export PUBLIC_DATADOG_CLIENT_TOKEN="${PUBLIC_DATADOG_CLIENT_TOKEN}"
              export PUBLIC_GIT_REV="${PUBLIC_GIT_REV}"
              export PUBLIC_LAST_MODIFIED_DATE="${PUBLIC_LAST_MODIFIED_DATE}"
              export PUBLIC_LAST_MODIFIED_EPOCH="${PUBLIC_LAST_MODIFIED_EPOCH}"
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
