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
          npmDepsHash = "sha256-L0uDljuA7K9+FLrVCqPQKZWW7q9ShgdRmYQBFipkSZQ=";
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
