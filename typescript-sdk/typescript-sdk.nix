{ ... }: {
  perSystem = { pkgs, nodePkgs, lib, ensureAtRepositoryRoot, ... }:
    let
      pkgsDeps = with pkgs; [ pkg-config ];
      nodeDeps = with nodePkgs; [ nodejs_21 ];
      combinedDeps = pkgsDeps ++ nodeDeps;
    in
    {
      packages = {
        typescript-sdk = nodePkgs.buildNpmPackage {
          npmDepsHash = "sha256-DVseAZQWLIeL74e9SRGkEJprm5OSZFUCWoUgmPEobpk=";
          src = ./.;
          pname = "@unionlabs/client";
          version = "0.0.0";
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
