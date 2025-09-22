_: {
  perSystem =
    {
      buildPnpmPackage,
      ...
    }:
    {
      packages = {
        ts-sdk-cosmos = buildPnpmPackage {
          packageJsonPath = ./package.json;
          extraSrcs = [
            ../ts-sdk
            ../ts-sdk-cosmos
          ];
          pnpmWorkspaces = [
            "@unionlabs/sdk"
            "@unionlabs/sdk-cosmos"
          ];
          hash = "sha256-X+yOSBK99AnS11sXHfQuQjqgjkxrCJms4z+A+Xrh8Ss=";
          doCheck = true;
          buildPhase = ''
            runHook preBuild
            pnpm --filter=@unionlabs/sdk-cosmos build
            runHook postBuild
          '';
          installPhase = ''
            mkdir -p $out
            cp -r ./ts-sdk-cosmos/build/* $out
          '';
          checkPhase = ''
            pnpm run --filter=@unionlabs/sdk-cosmos check
            pnpm run --filter=@unionlabs/sdk-cosmos test
          '';
        };
      };
      apps = { };
    };
}
