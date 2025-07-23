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
        sdk-evm = buildPnpmPackage {
          inherit pnpm;
          packageJsonPath = ./package.json;
          extraSrcs = [ ../ts-sdk ../sdk-evm ];
          pnpmWorkspaces = [ "@unionlabs/sdk" "@unionlabs/sdk-evm" ];
          #hash = "sha256-Qy3/L29jvGfev2eSeu7SkzYp8lUu5jaM7VzjcksoC4g=";
          doCheck = true;
          buildPhase = ''
            runHook preBuild
            pnpm --filter=@unionlabs/sdk-evm build
            runHook postBuild
          '';
          installPhase = ''
            mkdir -p $out
            cp -r ./sdk-evm/* $out
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
