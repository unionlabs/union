_: {
  perSystem =
    {
      buildPnpmPackage,
      pkgs,
      ...
    }:
    {
      packages = {
        ts-sdk-sui = buildPnpmPackage {
          packageJsonPath = ./package.json;
          extraSrcs = pkgs.lib.fileset.unions [
            ../ts-sdk
            ../ts-sdk-sui
          ];
          pnpmWorkspaces = [
            "@unionlabs/sdk"
            "@unionlabs/sdk-sui"
          ];
          hash = "sha256-o8Xi7tK7OzuMO0Kgbeu6iQbQP9Q3p8NBfWXFdrJVj9Y=";
          doCheck = true;
          buildPhase = ''
            runHook preBuild
            pnpm --filter=@unionlabs/sdk-sui build
            runHook postBuild
          '';
          installPhase = ''
            mkdir -p $out
            cp -r ./ts-sdk-sui/dist/* $out
          '';
          checkPhase = ''
            pnpm run --filter=@unionlabs/sdk-sui check
            pnpm run --filter=@unionlabs/sdk-sui test
          '';
        };
      };
      apps = { };
    };
}
