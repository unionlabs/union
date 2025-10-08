_: {
  perSystem =
    {
      pkgs,
      buildPnpmPackage,
      ...
    }:
    let
      hash = "sha256-oGq9J3PUuoYl2i1WZjyHXzfa7/SJKNA5apKVKNWV2qc=";
    in
    {
      packages = {
        ts-sdk-evm = buildPnpmPackage {
          inherit hash;
          packageJsonPath = ./package.json;
          extraSrcs = [
            ../ts-sdk
            ../ts-sdk-evm
          ];
          pnpmWorkspaces = [
            "@unionlabs/sdk"
            "@unionlabs/sdk-evm"
          ];
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
        ts-sdk-evm-docs = buildPnpmPackage {
          inherit hash;
          packageJsonPath = ./package.json;
          extraSrcs = [
            ../ts-sdk
            ../ts-sdk-evm
          ];
          pnpmWorkspaces = [
            "@unionlabs/sdk"
            "@unionlabs/sdk-evm"
          ];
          buildPhase = ''
            runHook preBuild
            pnpm --filter=@unionlabs/sdk-evm docgen
            runHook postBuild
          '';
          installPhase = ''
            mkdir -p $out
            cp -r ./ts-sdk-evm/docs/* $out
          '';
        };
      };
      apps = { };
    };
}
