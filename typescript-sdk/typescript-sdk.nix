{ ... }: {
  perSystem = { pkgs, unstablePkgs, lib, ensureAtRepositoryRoot, mkCi, ... }:
    let
      packageJSON = lib.importJSON ./package.json;
    in
    {
      packages = {
        typescript-sdk = unstablePkgs.buildNpmPackage {
          npmDepsHash = "sha256-6FK5jmTOxqEiLoy+T5ajD+ldzwXtVgSjkD6KPBO1dRk=";
          src = ./.;
          pname = packageJSON.name;
          version = packageJSON.version;
          doDist = false;
        };
      };
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
