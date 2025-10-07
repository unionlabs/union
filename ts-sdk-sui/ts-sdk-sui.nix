_: {
  perSystem =
    {
      pkgs,
      lib,
      ...
    }:
    let
      buildPnpmPackage = import ../tools/typescript/buildPnpmPackage.nix {
        inherit pkgs lib;
      };
      pnpm = pkgs.pnpm_10;
    in
    {
      packages = {
        ts-sdk-sui = buildPnpmPackage {
          inherit pnpm;
          packageJsonPath = ./package.json;
          extraSrcs = [
            ../ts-sdk
            ../ts-sdk-sui
          ];
          pnpmWorkspaces = [
            "@unionlabs/sdk"
            "@unionlabs/sdk-sui"
          ];
          hash = "sha256-nFzsUnmiZRyN0Gi3XT4W+srG7vJ8IsJ9wOfIdxb10NI=";
          doCheck = true;
          buildPhase = ''
            runHook preBuild
            pnpm --filter=@unionlabs/sdk-sui build
            runHook postBuild
          '';
          installPhase = ''
            mkdir -p $out
            cp -r ./ts-sdk-sui/* $out
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
