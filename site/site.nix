{ ... }: {
  perSystem = { pkgs, nodePkgs, lib, ensureAtRepositoryRoot, ... }:
    let
      pkgsDeps = with pkgs; [ pkg-config chromium ];
      nodeDeps = with nodePkgs; [ vips nodejs_21 ];
      combinedDeps = pkgsDeps ++ nodeDeps;
    in
    {
      packages = {
        site = nodePkgs.buildNpmPackage {
          npmDepsHash = "sha256-ePXWR/GNkE/2oT9nuFmWXkcxYBIrPMqtSbJj8aV1De8=";
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
          PUPPETEER_SKIP_DOWNLOAD = 1;
          PUPPETEER_EXECUTABLE_PATH = "${pkgs.chromium.outPath}/bin/chromium";
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

              export PUPPETEER_SKIP_DOWNLOAD=1 
              export PUPPETEER_EXECUTABLE_PATH="${pkgs.chromium.outPath}/bin/chromium";
              npm install
              npm run dev
            '';
          };
        };
      };
    };
}
