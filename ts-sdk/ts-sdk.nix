_: {
  perSystem =
    {
      pkgsUnstable,
      lib,
      self',
      ...
    }:
    let
      packageJson = lib.importJSON ./package.json;
    in
    {
      packages = {
        ts-sdk = pkgsUnstable.buildNpmPackage {
          pname = packageJson.name;
          inherit (packageJson) version;
          src = ./.;
          npmDepsHash = "sha256-z8Nnv3AnXcA0NBtKtyCAU/LQ18vRQykTvrb5J++ufOI=";
          doCheck = true;
          checkPhase = ''
            npm run test
          '';
        };

      };
      apps.publish-ts-sdk = {
        type = "app";
        program = pkgsUnstable.writeShellApplication {
          name = "publish-ts-sdk";
          text = ''
            cd ${self'.packages.ts-sdk}/lib/node_modules/@unionlabs/sdk
            ${pkgsUnstable.nodejs}/bin/npm publish --access='public' --no-git-tagsh
          '';
        };

      };
    };
}
