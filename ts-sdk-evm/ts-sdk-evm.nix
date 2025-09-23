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
          hash = "sha256-/2HUXozGfMaviwZaBKcUuy9Njq6wh3XVzT4sVbT7Z/8=";
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
