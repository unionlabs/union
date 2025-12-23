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
        ceremony = buildPnpmPackage {
          hash = "sha256-1R/7+vIllXY+KmNgxxH2fEHM4UzVmHDiDA3XqmODuJI=";
          packageJsonPath = ./package.json;
          extraSrcs = [
            ../ceremony
          ];
          pnpmWorkspaces = [
            "union-cermony"
          ];
          buildPhase = ''
            runHook preBuild
            export NODE_OPTIONS="--no-warnings";
            export VITE_BUCKET_ID="contributions";
            export VITE_SUPABASE_URL="https://otfaamdxmgnkjqsosxye.supabase.co/";
            export VITE_SUPABASE_ANON_KEY="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im90ZmFhbWR4bWdua2pxc29zeHllIiwicm9sZSI6ImFub24iLCJpYXQiOjE3MjEzMjA5NDMsImV4cCI6MjAzNjg5Njk0M30.q91NJPFFHKJXnbhbpUYwsB0NmimtD7pGPx6PkbB_A3w";
            pnpm --filter=union-cermony build
            runHook postBuild
          '';
          installPhase = ''
            mkdir -p $out
            cp -r ./ceremony/build/* $out
          '';
          doDist = false;
        };
      };

      apps = {
        ceremony-dev-server = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "ceremony-dev-server";
            text = ''
              ${ensureAtRepositoryRoot}
              cd ceremony/

              export NODE_OPTIONS="--no-warnings"
              export VITE_BUCKET_ID="contributions"
              export VITE_SUPABASE_URL="https://otfaamdxmgnkjqsosxye.supabase.co/"
              export VITE_SUPABASE_ANON_KEY="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im90ZmFhbWR4bWdua2pxc29zeHllIiwicm9sZSI6ImFub24iLCJpYXQiOjE3MjEzMjA5NDMsImV4cCI6MjAzNjg5Njk0M30.q91NJPFFHKJXnbhbpUYwsB0NmimtD7pGPx6PkbB_A3w"

              pnpm install
              pnpm run dev -- --host
            '';
          };
        };
      };
    };
}
