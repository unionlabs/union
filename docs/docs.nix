{ ... }: {
  perSystem = { pkgs, lib, ... }:
    let
      npmDepsHash = "sha256-Xj8s7tNjZcbq474jitfq0sII50gy6nj9Bwrs09+9Zqc=";
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
          srcs = ./. ../unionvisor/docs;
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
