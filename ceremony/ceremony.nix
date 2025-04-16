_: {
  perSystem =
    {
      lib,
      pkgs,
      pkgsUnstable,
      ensureAtRepositoryRoot,
      ...
    }:
    let
      deps = with pkgsUnstable; [
        pkg-config
        python3
        nodePackages_latest.nodejs
      ];
      packageJSON = lib.importJSON ./package.json;
    in
    {
      packages = {
        ceremony = pkgsUnstable.buildNpmPackage {
          npmDepsHash = "sha256-2s556is4PJaGmxQhIUPZLINh6J6ngeTf32bwDtl+v6Q=";
          src = ./.;
          sourceRoot = "ceremony";
          npmFlags = [ "--legacy-peer-deps" ];
          pname = packageJSON.name;
          inherit (packageJSON) version;
          nativeBuildInputs = deps;
          buildInputs = deps;
          installPhase = ''
            mkdir -p $out
            cp -r ./build/* $out
          '';
          doDist = false;
          NODE_OPTIONS = "--no-warnings";
          VITE_BUCKET_ID = "contributions";
          VITE_SUPABASE_URL = "https://otfaamdxmgnkjqsosxye.supabase.co/";
          VITE_SUPABASE_ANON_KEY = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im90ZmFhbWR4bWdua2pxc29zeHllIiwicm9sZSI6ImFub24iLCJpYXQiOjE3MjEzMjA5NDMsImV4cCI6MjAzNjg5Njk0M30.q91NJPFFHKJXnbhbpUYwsB0NmimtD7pGPx6PkbB_A3w";
        };
      };

      apps = {
        ceremony-dev-server = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "ceremony-dev-server";
            runtimeInputs = deps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd ceremony/

              export NODE_OPTIONS="--no-warnings"
              export VITE_BUCKET_ID="contributions"
              export VITE_SUPABASE_URL="https://otfaamdxmgnkjqsosxye.supabase.co/"
              export VITE_SUPABASE_ANON_KEY="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im90ZmFhbWR4bWdua2pxc29zeHllIiwicm9sZSI6ImFub24iLCJpYXQiOjE3MjEzMjA5NDMsImV4cCI6MjAzNjg5Njk0M30.q91NJPFFHKJXnbhbpUYwsB0NmimtD7pGPx6PkbB_A3w"

              npm install
              npm run dev -- --host
            '';
          };
        };
      };
    };
}
