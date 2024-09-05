{ ... }: {
  perSystem = { pkgs, unstablePkgs, lib, ensureAtRepositoryRoot, ... }:
    let
      pkgsDeps = with pkgs; [ pkg-config python3 ];
      nodeDeps = with unstablePkgs; [ nodePackages_latest.nodejs ];
      combinedDeps = pkgsDeps ++ nodeDeps;
      packageJSON = lib.importJSON ./package.json;
    in
    {
      packages = {
        ceremony = unstablePkgs.buildNpmPackage {
          npmDepsHash = "sha256-0cbA3OQZDfl+LZlgSAi8PgmK0Ln7VMoqWSdN+lqX6w0=";
          src = ./.;
          sourceRoot = "ceremony";
          npmFlags = [ "--legacy-peer-deps" ];
          pname = packageJSON.name;
          version = packageJSON.version;
          nativeBuildInputs = combinedDeps;
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
        ceremony-dev-server = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "ceremony-dev-server";
            runtimeInputs = combinedDeps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd ceremony/

              npm install
              npm run dev
            '';
          };
        };
      };
    };
}
