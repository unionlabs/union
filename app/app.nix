{ ... }: {
  perSystem = { pkgs, nodePkgs, lib, ensureAtRepositoryRoot, ... }:
    let
      pkgsDeps = with pkgs; [ pkg-config ];
      nodeDeps = with nodePkgs; [ nodejs_21 ];
      combinedDeps = pkgsDeps ++ nodeDeps;
      packageJSON = lib.importJSON ./package.json;
    in
    {
      packages = {
        app = nodePkgs.buildNpmPackage {
          npmDepsHash = "sha256-hx5y0HrREkKPfHU02CR32ZvLaPOHmGEtqPUluTuFXwo=";
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
