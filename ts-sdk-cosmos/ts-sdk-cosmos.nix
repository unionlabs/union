_: {
  perSystem =
    {
      buildPnpmPackage,
      ...
    }:
    let
      hash = "sha256-8fBgRR9gTZRsImkeRxVkzADjQb3DH7+KSi+MMvNdI00=";
    in
    {
      packages = {
        ts-sdk-cosmos = buildPnpmPackage {
          inherit hash;
          packageJsonPath = ./package.json;
          extraSrcs = [
            ../ts-sdk
            ../ts-sdk-cosmos
          ];
          pnpmWorkspaces = [
            "@unionlabs/sdk"
            "@unionlabs/sdk-cosmos"
          ];
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
        ts-sdk-cosmos-docs = buildPnpmPackage {
          inherit hash;
          packageJsonPath = ./package.json;
          extraSrcs = [
            ../ts-sdk
            ../ts-sdk-cosmos
          ];
          pnpmWorkspaces = [
            "@unionlabs/sdk"
            "@unionlabs/sdk-cosmos"
          ];
          buildPhase = ''
            runHook preBuild
            pnpm --filter=@unionlabs/sdk-cosmos docgen
            runHook postBuild
          '';
          installPhase = ''
            mkdir -p $out
            cp -r ./ts-sdk-cosmos/docs/* $out
          '';
        };
      };
      apps = { };
    };
}
