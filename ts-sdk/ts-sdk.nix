_: {
  perSystem =
    {
      pkgs,
      lib,
      self',
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
        ts-sdk = buildPnpmPackage {
          inherit pnpm;
          packageJsonPath = ./package.json;
          extraSrcs = [ ../ts-sdk ];
          pnpmWorkspaces = [ "@unionlabs/sdk" ];
          hash = "sha256-wgW2jT+ujgtWptgKo9bQelR9sM99UF9BaXblfv6/R7I=";
          doCheck = true;
          buildPhase = ''
            runHook preBuild
            pnpm --filter=@unionlabs/sdk build
            runHook postBuild
          '';
          installPhase = ''
            mkdir -p $out
            cp -r ./ts-sdk/* $out
          '';
          checkPhase = ''
            pnpm run --filter=@unionlabs/sdk test
          '';
        };
      };
      apps = {
        publish-ts-sdk = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "publish-ts-sdk";
            text = ''
              cd ${self'.packages.ts-sdk}/
              ${pnpm} publish --access='public'
            '';
          };
        };
      };
    };
}
