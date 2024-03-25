{ ... }: {
  perSystem = { pkgs, unstablePkgs, lib, ensureAtRepositoryRoot, ... }:
    let
      pkgsDeps = with pkgs; [ pkg-config ];
      nodeDeps = with unstablePkgs; [ nodePackages_latest.nodejs ];
      combinedDeps = pkgsDeps ++ nodeDeps;
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
