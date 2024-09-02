{ ... }: {
  perSystem = { pkgs, unstablePkgs, lib, ensureAtRepositoryRoot, mkCi, ... }:
    {
      apps = {
        ts-sdk-check = {
          type = "app";
          program = unstablePkgs.writeShellApplication {
            name = "ts-sdk-check";
            text = ''
              ${ensureAtRepositoryRoot}
              biome check typescript-sdk --error-on-warnings --write --unsafe

              cd typescript-sdk
              bun run typecheck
              
              nix fmt

              nix build .\#checks.${pkgs.system}.spellcheck --print-build-logs
            '';
          };
        };
      };
    };
}
