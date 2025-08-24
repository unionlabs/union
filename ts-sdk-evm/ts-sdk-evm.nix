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
        ts-sdk-evm = buildPnpmPackage {
          inherit pnpm;
          packageJsonPath = ./package.json;
          extraSrcs = [
            ../ts-sdk
            ../ts-sdk-evm
          ];
          pnpmWorkspaces = [
            "@unionlabs/sdk"
            "@unionlabs/sdk-evm"
          ];
          hash = "sha256-8o8tWaSBVFNGTwsDNnNQZbeugjH8KrPW2uyE3wMNe8w=";
          doCheck = true;
          buildPhase = ''
            runHook preBuild
            pnpm --filter=@unionlabs/sdk-evm build
            runHook postBuild
          '';
          installPhase = ''
            mkdir -p $out
            cp -r ./ts-sdk-evm/* $out
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
