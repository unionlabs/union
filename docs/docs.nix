{ ... }: {
  perSystem = { pkgs, ... }: {
    checks =
      {
        docs = pkgs.buildNpmPackage {
          name = "docs";
          src = ./.;
          buildPhase = ''
            npm run build
          '';
          npmDepsHash = "sha256-j/i0MM+kzvcsZs8aWab6xdHJ+QSW0S1MQcS+A2RiTY0=";
        };

      };

    packages = {
      docs = pkgs.buildNpmPackage {
        name = "docs";
        src = ./.;
        buildPhase = ''
          npm run build
        '';
        npmDepsHash = "sha256-j/i0MM+kzvcsZs8aWab6xdHJ+QSW0S1MQcS+A2RiTY0=";
        installPhase = ''
          mkdir -p $out
          cp -dR ./build $out
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
