{ ... }: {
  perSystem = { pkgs, lib, ensureAtRepositoryRoot, ... }:
    let pkgsDeps = with pkgs; [ nodejs_20 pkg-config ];
    in {
      packages = {
        app = pkgs.buildNpmPackage {
          npmDepsHash = "sha256-lNtno0vO/BHoQRFZh4uzl1bPNj6p5zUomTO7/ClnGkw=";
          src = ./.;
          sourceRoot = "app";
          pname = "app";
          version = "0.0.0";
          nativeBuildInputs = pkgsDeps ++ [ pkgs.python3 ];
          buildInputs = pkgsDeps;
          installPhase = ''
            mkdir -p $out
            cp -r ./build/* $out
          '';
          doDist = false;
        };
      };

      apps = {
        app-dev-server = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "app-dev-server";
            runtimeInputs = pkgsDeps;
            text = ''
              ${ensureAtRepositoryRoot}
              cd app/

              npm install
              npm run dev
            '';
          };
        };
      };
    };
}
