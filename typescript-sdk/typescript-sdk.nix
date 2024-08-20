{ ... }: {
  perSystem = { pkgs, unstablePkgs, lib, ensureAtRepositoryRoot, ... }:
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
    };
}
