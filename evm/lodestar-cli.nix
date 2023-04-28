{ ... }: {
  perSystem = { self', pkgs, ... }: {
    packages.lodestar-cli =
      let
        src = pkgs.fetchFromGitHub {
          owner = "chainsafe";
          repo = "lodestar";
          rev = "c65e1a428c43f252b99a5fffa77fbc27f224dc07";
          hash = "sha256-Y8Jv6WV066Q0osfNc9+HnwikrZrrtwBzLCz+t3M69bA=";
        };
        nodePackage = pkgs.stdenv.mkDerivation {
          __noChroot = true;
          name = "lodestar-node";
          version = "v1.6.0";
          nativeBuildInputs = with pkgs; [
            python3
            nodejs
            nodePackages.node-gyp-build
            yarn
            (snappy.override { static = true; })
            pkg-config
          ];
          inherit src;
          buildPhase = ''
            export HOME=$(mktemp -d)
            yarn
            export PATH="$(pwd)/node_modules/.bin:$PATH"
            yarn run build
          '';
          installPhase = ''
            mkdir -p $out
            cp -R . $out
          '';
        };
      in
      pkgs.writeShellApplication {
        name = "lodestar-cli";
        runtimeInputs = [ pkgs.nodejs ];
        text = ''
          ${nodePackage}/packages/cli/bin/lodestar.js "$@"
        '';
      };
  };
}

