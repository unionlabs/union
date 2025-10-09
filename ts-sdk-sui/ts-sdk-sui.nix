_: {
  perSystem =
    {
      buildPnpmPackage,
      ...
    }:
    {
      packages = {
        ts-sdk-sui = buildPnpmPackage {
          packageJsonPath = ./package.json;
          extraSrcs = [
            ../ts-sdk
            ../ts-sdk-sui
          ];
          pnpmWorkspaces = [
            "@unionlabs/sdk"
            "@unionlabs/sdk-sui"
          ];
          hash = "sha256-8fBgRR9gTZRsImkeRxVkzADjQb3DH7+KSi+MMvNdI00=";
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
