_: {
  perSystem =
    {
      pkgs,
      pkgsUnstable,
      ensureAtRepositoryRoot,
      buildPnpmPackage,
      ...
    }:
    let
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
        explorer = buildPnpmPackage rec {
          packageJsonPath = ./package.json;
          extraSrcs = pkgs.lib.fileset.unions [
            ../explorer
            ../effect-svelte
            ../ts-sdk
            ../ts-sdk-cosmos
            ../ts-sdk-evm
            ../ts-sdk-sui
          ];
          hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
          buildInputs = deps;
          nativeBuildInputs = buildInputs;
          pnpmWorkspaces = [
            "explorer"
            "@unionlabs/effect-svelte"
            "@unionlabs/sdk"
            "@unionlabs/sdk-cosmos"
            "@unionlabs/sdk-evm"
            "@unionlabs/sdk-sui"
          ];
          buildPhase = ''
            runHook preBuild
            pnpm --filter=explorer --filter=effect-svelte prepare
            pnpm --filter=explorer build
            runHook postBuild
          '';
          checkPhase = ''
            pnpm --filter=explorer check
          '';
          doCheck = true;
          installPhase = ''
            mkdir -p $out
            cp -r ./explorer/build/* $out
          '';
          doDist = false;
        };
      };
      apps = {
        explorer-dev-server = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "explorer-dev-server";
            runtimeInputs = deps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd explorer/
              pnpm install
              pnpm run dev --host
            '';
          };
        };
        explorer-check-watch = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "explorer-check-watch";
            runtimeInputs = deps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd explorer/
              pnpm run check --watch --threshold error
            '';
          };
        };
      };
    };
}
