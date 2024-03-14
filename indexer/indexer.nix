{ ... }: {
  perSystem = { pkgs, unstablePkgs, lib, ensureAtRepositoryRoot, ... }:
    let
      pkgsDeps = with pkgs; [ pkg-config ];
      nodeDeps = with unstablePkgs; [ nodejs_21 ];
      combinedDeps = pkgsDeps ++ nodeDeps;
      packageJSON = lib.importJSON ./package.json;
    in
    {
      packages = {
        indexer = unstablePkgs.buildNpmPackage {
          npmDepsHash = "sha256-rG1hz40XqA9YH0Bk8sVc9uK7C8SHulLLFFzMDIuroc8=";
          src = ./.;
          sourceRoot = "indexer";
          pname = packageJSON.name;
          version = packageJSON.version;
          nativeBuildInputs = combinedDeps;
          buildInputs = combinedDeps;
          dontNpmBuild = true;
          installPhase = ''
            mkdir -p $out
            cp -r ./* $out
          '';
          doDist = false;
          NODE_OPTIONS = "--no-warnings";
          PONDER_TELEMETRY_DISABLED = true;
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
