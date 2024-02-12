{ ... }: {
  perSystem = { pkgs, nodePkgs, lib, ensureAtRepositoryRoot, ... }:
    let
      pkgsDeps = with pkgs; [ pkg-config ];
      nodeDeps = with nodePkgs; [ vips nodejs_21 ];
      combinedDeps = pkgsDeps ++ nodeDeps;
    in
    {
      packages = {
        site = nodePkgs.buildNpmPackage {
          npmDepsHash = "sha256-s5d0EmDQhQ8G5GVpOGY1M7Y2OLkpnMpjYqMl1LbtljE=";
          src = ./.;
          srcs = [ ./. ./../evm/. ./../networks/genesis/. ./../versions/. ];
          sourceRoot = "site";
          pname = "site";
          version = "0.0.1";
          nativeBuildInputs = combinedDeps;
          buildInputs = combinedDeps;
          installPhase = ''
            mkdir -p $out
            cp -r ./dist/* $out
          '';
          doDist = false;
        };
      };

      apps = {
        site-dev-server = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "site-dev-server";
            runtimeInputs = combinedDeps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd site/

              npm install
              npm run dev
            '';
          };
        };
      };
    };
}
