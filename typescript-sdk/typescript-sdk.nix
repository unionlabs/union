{ ... }: {
  perSystem = { pkgs, unstablePkgs, lib, ensureAtRepositoryRoot, ... }:
    let
      pkgsDeps = with pkgs; [ pkg-config ];
      nodeDeps = with unstablePkgs; [ nodePackages_latest.nodejs ];
      combinedDeps = pkgsDeps ++ nodeDeps;
      packageJSON = lib.importJSON ./package.json;
    in
    {
      packages = { };

      apps = {
        typescript-sdk-check = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "typescript-sdk-check";
            runtimeInputs = combinedDeps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd typescript-sdk/
              
              node_modules/.bin/tsc --project ./tsconfig.json
            '';
          };
        };
      };
    };
}
