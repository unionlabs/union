{ ... }: {
  perSystem = { pkgs, unstablePkgs, lib, ensureAtRepositoryRoot, ... }:
    let
      pkgsDeps = with pkgs; [ pkg-config ];
      nodeDeps = with unstablePkgs; [ nodejs_21 ];
      combinedDeps = pkgsDeps ++ nodeDeps;
      packageJSON = lib.importJSON ./package.json;
    in
    {
      packages = {
        typescript-sdk = unstablePkgs.buildNpmPackage {
          npmDepsHash = "sha256-gERPCJE54DLGc0LSqlFLxCeC8mI5W2mCJcnaO6fkFmk=";
          src = ./.;
          pname = packageJSON.name;
          version = packageJSON.version;
          nativeBuildInputs = combinedDeps;
          buildInputs = combinedDeps;
          installPhase = ''
            mkdir -p $out
            cp -r ./dist/* $out
          '';
          doDist = false;
        };
      };
    };
}
