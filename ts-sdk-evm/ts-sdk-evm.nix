_: {
  perSystem =
    {
      buildPnpmPackage,
      ...
    }:
    {
      packages = {
        ts-sdk-evm = buildPnpmPackage {
          packageJsonPath = ./package.json;
          extraSrcs = [
            ../ts-sdk
            ../ts-sdk-evm
          ];
          pnpmWorkspaces = [
            "@unionlabs/sdk"
            "@unionlabs/sdk-evm"
          ];
          hash = "sha256-uvNz7xiDzbiP8lnWAPOJyDT79bkLhdZepaSrZ0u4260=";
          doCheck = true;
          buildPhase = ''
            runHook preBuild
            pnpm --filter=@unionlabs/sdk-evm build
            runHook postBuild
          '';
          installPhase = ''
            mkdir -p $out
            cp -r ./ts-sdk-evm/build/* $out
          '';
          checkPhase = ''
            pnpm run --filter=@unionlabs/sdk-evm check
            pnpm run --filter=@unionlabs/sdk-evm test
          '';
        };
      };
      apps = { };
    };
}
