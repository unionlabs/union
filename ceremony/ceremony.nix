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
          npmDepsHash = "sha256-/vwKKNOjLFo6tlOHXeaE8sDOvjm2B2APNw+5g1AdTZk=";
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
          VITE_BUCKET_ID="contributions";
          VITE_SUPABASE_URL="https://otfaamdxmgnkjqsosxye.supabase.co/";
          VITE_SUPABASE_ANON_KEY="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im90ZmFhbWR4bWdua2pxc29zeHllIiwicm9sZSI6ImFub24iLCJpYXQiOjE3MjEzMjA5NDMsImV4cCI6MjAzNjg5Njk0M30.q91NJPFFHKJXnbhbpUYwsB0NmimtD7pGPx6PkbB_A3w";
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
