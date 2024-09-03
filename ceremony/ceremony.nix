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
          VITE_SUPABASE_URL = "https://wwqpylbrcpriyaqugzsi.supabase.co";
          VITE_SUPABASE_ANON_KEY="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Ind3cXB5bGJyY3ByaXlhcXVnenNpIiwicm9sZSI6ImFub24iLCJpYXQiOjE3MjI0MzMyMjQsImV4cCI6MjAzODAwOTIyNH0.UQOmQ-wE63O32lyrLDO7ryowrM5LNA2UILHDA7hTH8E";
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
