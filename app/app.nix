{ ... }: {
  perSystem = { pkgs, nodePkgs, lib, ensureAtRepositoryRoot, ... }:
    let
      pkgsDeps = with pkgs; [ pkg-config ];
      nodeDeps = with nodePkgs; [ nodejs_21 ];
      combinedDeps = pkgsDeps ++ nodeDeps;
    in
    {
      packages = {
        app = nodePkgs.buildNpmPackage {
          npmDepsHash = "sha256-37MrESqNHncyDJg/Qpdl6xcm7SEoMRgWq2Amm5t04Jk=";
          src = ./.;
          sourceRoot = "app";
          pname = "app";
          version = "0.0.0";
          nativeBuildInputs = combinedDeps ++ [ pkgs.python3 ];
          buildInputs = combinedDeps;
          installPhase = ''
            mkdir -p $out
            cp -r ./build/* $out
          '';
          doDist = false;
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
