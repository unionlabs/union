_: {
  perSystem =
    {
      pkgs,
      ensureAtRepositoryRoot,
      buildPnpmPackage,
      ...
    }:
    {
      packages = {
        zkgm-dev = buildPnpmPackage {
          hash = "sha256-zwQF3zKQSD5WFjn7EhmjNwVWW0NytYMZlHoc+aJ5/9M=";
          packageJsonPath = ./package.json;
          extraSrcs = [
            ../zkgm-dev
          ];
          pnpmWorkspaces = [
            "zkgm-dev"
          ];
          buildPhase = ''
            runHook preBuild
            export NODE_OPTIONS="--no-warnings";
            export VITE_BUCKET_ID="contributions";
            export VITE_SUPABASE_URL="https://otfaamdxmgnkjqsosxye.supabase.co/";
            export VITE_SUPABASE_ANON_KEY="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im90ZmFhbWR4bWdua2pxc29zeHllIiwicm9sZSI6ImFub24iLCJpYXQiOjE3MjEzMjA5NDMsImV4cCI6MjAzNjg5Njk0M30.q91NJPFFHKJXnbhbpUYwsB0NmimtD7pGPx6PkbB_A3w";
            pnpm --filter=zkgm-dev build
            runHook postBuild
          '';
          installPhase = ''
            mkdir -p $out
            cp -r ./zkgm-dev/build/* $out
          '';
          doDist = false;
        };
      };

      apps = {
        zkgm-dev-dev-server = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "zkgm-dev-dev-server";
            text = ''
              ${ensureAtRepositoryRoot}
              cd zkgm-dev/

              export NODE_OPTIONS="--no-warnings"
              export VITE_BUCKET_ID="contributions"
              export VITE_SUPABASE_URL="https://otfaamdxmgnkjqsosxye.supabase.co/"
              export VITE_SUPABASE_ANON_KEY="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im90ZmFhbWR4bWdua2pxc29zeHllIiwicm9sZSI6ImFub24iLCJpYXQiOjE3MjEzMjA5NDMsImV4cCI6MjAzNjg5Njk0M30.q91NJPFFHKJXnbhbpUYwsB0NmimtD7pGPx6PkbB_A3w"

              pnpm install
              npm run dev -- --host
            '';
          };
        };
      };
    };
}
