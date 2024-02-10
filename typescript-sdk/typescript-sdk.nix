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
          npmDepsHash = "sha256-84gQgCGqNm/zBD+gy4bCbuEyoqsAaueoKI/LX2ofJ3w=";
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
