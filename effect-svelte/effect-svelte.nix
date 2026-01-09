_: {
  perSystem =
    {
      pkgs,
      buildPnpmPackage,
      self',
      ...
    }:
    {
      packages = {
        effect-svelte = buildPnpmPackage {
          packageJsonPath = ./package.json;
          extraSrcs = pkgs.lib.fileset.unions [ ./. ];
          pnpmWorkspaces = [ "@unionlabs/effect-svelte" ];
          hash = "sha256-q+7B/VoFaV85Mt2hQWJBFIEQH5zVdwoESqgqhMxKCJU=";
          doCheck = true;
          buildPhase = ''
            runHook preBuild
            pnpm --filter=@unionlabs/effect-svelte build
            (
              cd effect-svelte
              pnpm pack
            )
            runHook postBuild
          '';
          installPhase = ''
            mkdir -p $out
            cp ./effect-svelte/unionlabs-effect-svelte-*.tgz $out
          '';
          checkPhase = ''
            pnpm --filter=@unionlabs/effect-svelte check
            pnpm --filter=@unionlabs/effect-svelte test
          '';
        };
      };
      apps = {
        publish-effect-svelte = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "publish-effect-svelte";
            text = ''
              cd ${self'.packages.effect-svelte}/
              ${pkgs.pnpm}/bin/pnpm publish --access='public'
            '';
          };
        };
      };
    };
}
