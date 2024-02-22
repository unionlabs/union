{ ... }: {
  perSystem = { pkgs, nodePkgs, lib, ensureAtRepositoryRoot, ... }:
    let
      pkgsDeps = with pkgs; [ pkg-config ];
      nodeDeps = with nodePkgs; [ nodejs_21 ];
      combinedDeps = pkgsDeps ++ nodeDeps;
    in
    {
      packages = {
        indexer = nodePkgs.buildNpmPackage {
          npmDepsHash = "sha256-3P5MJUo1o+dQL1pwDfSMSnnKzdSONKkPA5IUrS7CAFI=";
          src = ./.;
          sourceRoot = "indexer";
          pname = "union-transfers-indexer";
          version = "0.0.0";
          nativeBuildInputs = combinedDeps;
          buildInputs = combinedDeps;
          dontNpmBuild = true;
          installPhase = ''
            mkdir -p $out
            cp -r ./* $out
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
