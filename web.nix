_: {
  perSystem =
    {
      pkgs,
      pkgsUnstable,
      ensureAtRepositoryRoot,
      lib,
      self',
      ...
    }:
    let
      pnpm = pkgsUnstable.pnpm_10;
      app2Deps = with pkgsUnstable; [
        python3
        stdenv.cc
        pkg-config
        nodePackages_latest.nodejs
        pnpm_10
      ];
      mkPackage =
        {
          packageJsonPath,
          src ? ./.,
          pnpmWorkspaces ? [ ],
          hash,
        }:
        let
          packageJson = lib.importJSON packageJsonPath;
          pname = packageJson.name;
          inherit (packageJson) version;
          pnpmDeps = pnpm.fetchDeps {
            inherit
              pname
              version
              src
              hash
              pnpmWorkspaces
              ;
          };
        in
        {
          inherit
            pnpmDeps
            pname
            version
            pnpmWorkspaces
            src
            ;
          npmDeps = pnpmDeps;
          npmConfigHook = pnpm.configHook;
        };
    in
    {
      packages = {
        app2 = pkgsUnstable.buildNpmPackage (
          (mkPackage {
            packageJsonPath = ./app2/package.json;
            hash = "sha256-Fl/wNs8d6s4mrjMrQJy+uSXJ6Jmf7RllHKXFr8vqhv8=";
          })
          // rec {
            buildInputs = app2Deps;
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
          }
        );
        ts-sdk = pkgsUnstable.buildNpmPackage (
          (mkPackage {
            packageJsonPath = ./ts-sdk/package.json;
            hash = "sha256-d0z7ud47Z8XoCw7wOgnXKR9siT1TFw87lACSUJev5Xo=";
            pnpmWorkspaces = [ "@unionlabs/sdk" ];
          })
          // {
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
          }
        );
      };
      apps = {
        app2-dev-server = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "app-dev-server";
            runtimeInputs = app2Deps;
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
            runtimeInputs = app2Deps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd app2/
              pnpm dlx gql.tada generate-schema --tsconfig ./tsconfig.json --output "./src/generated/schema.graphql" "https://development.graphql.union.build/v1/graphql"
              pnpm dlx gql.tada generate-output --disable-preprocessing --tsconfig ./tsconfig.json --output ./src/generated/graphql-env.d.ts
            '';
          };
        };
        publish-ts-sdk = {
          type = "app";
          program = pkgsUnstable.writeShellApplication {
            name = "publish-ts-sdk";
            text = ''
              cd ${self'.packages.ts-sdk}/
              ${pnpm} publish --dry-run
            '';
          };
        };
      };
    };
}
