{ ... }: {
  perSystem = { pkgs, lib, ... }:
    let
      npmDepsHash = "sha256-A/8NpoJTWi6ylnrkpCwekfXbPH82jNdIA8Z58jMbYQI=";
    in
    {
      checks =
        {
          docs = pkgs.buildNpmPackage {
            name = "docs";
            src = ./.;
            buildPhase = ''
              npm run build
            '';
            npmDepsHash = npmDepsHash;
          };

        };

      packages = {
        docs = pkgs.buildNpmPackage {
          name = "docs";
          src = ./.;
          buildPhase = ''
            npm run build
          '';
          npmDepsHash = npmDepsHash;
          installPhase = ''
            mkdir -p $out
            cp -dR ./build/* $out
            echo "built docs"
          '';
        };
      };

      apps = {
        docs-dev-server = {
          type = "app";
          program = pkgs.writeShellApplication {
            name = "docs-dev-server";
            runtimeInputs = [ pkgs.nodejs ];
            text = ''
              cd docs
              npm install
              npm run start
            '';
          };
        };
      };
    };
}
