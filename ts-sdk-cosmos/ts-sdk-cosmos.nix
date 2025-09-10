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
        ts-sdk-cosmos = buildPnpmPackage {
          inherit pnpm;
          packageJsonPath = ./package.json;
          extraSrcs = [
            ../ts-sdk
            ../ts-sdk-cosmos
          ];
          pnpmWorkspaces = [
            "@unionlabs/sdk"
            "@unionlabs/sdk-cosmos"
          ];
          hash = "sha256-vrjzTdwKSucWWetnL4kw2KLCS9NhXfkiM9LgoswD2QE=";
          doCheck = true;
          buildPhase = ''
            runHook preBuild
            pnpm --filter=@unionlabs/sdk-cosmos build
            runHook postBuild
          '';
          installPhase = ''
            mkdir -p $out
            cp -r ./ts-sdk-cosmos/* $out
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
