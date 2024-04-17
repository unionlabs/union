{ ... }: {
  perSystem = { pkgs, javascriptPkgs, lib, ensureAtRepositoryRoot, ... }:
    let
      pkgsDeps = with pkgs; [ pkg-config ];
      nodeDeps = with javascriptPkgs; [ nodejs_21 ];
      combinedDeps = pkgsDeps ++ nodeDeps;
      packageJSON = lib.importJSON ./package.json;
    in
    {
      packages = {
        app = javascriptPkgs.buildNpmPackage {
          npmDepsHash = "sha256-yZ1tygnZX07qovPOGK4sF0uVCxyVS4qdfMUStVvVFrs=";
          src = ./.;
          sourceRoot = "app";
          pname = packageJSON.name;
          version = packageJSON.version;
          nativeBuildInputs = combinedDeps ++ [ pkgs.python3 ];
          buildInputs = combinedDeps;
          installPhase = ''
            mkdir -p $out
            cp -r ./build/* $out
          '';
          doDist = false;
          NODE_OPTIONS = "--no-warnings";
        };
      };

      apps = {
        app-dev-server = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "app-dev-server";
            runtimeInputs = combinedDeps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd app/

              npm install
              npm run dev
            '';
          };
        };
      };
    };
}
