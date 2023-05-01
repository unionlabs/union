{ ... }: {
  perSystem = { self', pkgs, ... }: {
    packages.generate-evm-proto =
      let
        solidity-protobuf = pkgs.stdenv.mkDerivation {
          name = "solidity-protobuf";
          version = "0.0.1";
          src = pkgs.fetchFromGitHub {
            owner = "CyrusVorwald";
            repo = "solidity-protobuf";
            rev = "1c323bed92d373d6c4d6c728c8dd9f76cf4b5a0c";
            hash = "sha256-1obEhMjaLToaSk920CiJwfhkw+LDgY5Y/b7SpkeuqDE=";
          };
          buildInputs = [
            (pkgs.python3.withPackages (ps: with ps; [ protobuf wrapt ]))
          ];
          buildPhase = "true";
          installPhase = ''
            mkdir $out
            cp -r $src/* $out
          '';
        };
        cometbls-src = builtins.fetchGit {
          url = "git@github.com:UnionFi/cometbls.git";
          rev = "f19ae296cf176b343ea214967810ba735813e73f";
        };
        cosmossdk-src = builtins.fetchGit {
          url = "git@github.com:UnionFi/cosmos-sdk.git";
          rev = "021566a5aba49e79356e2e6e246494e118f12605";
        };
        ibcgo-src = pkgs.fetchFromGitHub {
          owner = "strangelove-ventures";
          repo = "ibc-go";
          rev = "f8081a1828e47e11791b036659dd6d0e7be5473b";
          sha256 = "sha256-e9z9+VxoQkrvWeYzdxHax6L10eQebRjW7GrD5wnaLv8=";
        };
        ics23-src = pkgs.fetchFromGitHub {
          owner = "cosmos";
          repo = "ics23";
          rev = "b1abd8678aab07165efd453c96796a179eb3131f";
          sha256 = "sha256-O7oZI+29xKAbMHssg5HhxlssedSfejCuzHNHYX7WwBc=";
        };
        cosmosproto-src = pkgs.fetchFromGitHub {
          owner = "cosmos";
          repo = "cosmos-proto";
          rev = "v1.0.0-beta.3";
          sha256 = "sha256-kFm1ChSmm5pU9oJqKmWq4KfO/hxgxzvcSzr66oTulos=";
        };
        gogoproto-src = pkgs.fetchFromGitHub {
          owner = "cosmos";
          repo = "gogoproto";
          rev = "v1.4.7";
          sha256 = "sha256-oaGwDFbz/xgL7hDtvdh/mIcRIGBdp+/xuKeuBE2ZpqY=";
        };
        googleapis-src = pkgs.fetchFromGitHub {
          owner = "googleapis";
          repo = "googleapis";
          rev = "6774ccbbc3f182f6ae3a32dca29e1da489ad8a8f";
          sha256 = "sha256-TME4wkdmqrb0Shuc5uFqSGSoDaMhM9YJv9kvTam7c9I=";
        };
      in
      pkgs.writeShellApplication {
        name = "generate-evm-proto";
        runtimeInputs = [ pkgs.protobuf ];
        text = ''
                  plugindir="${solidity-protobuf}/protobuf-solidity/src/protoc"
                  find ${ibcgo-src}/proto -name "$1" |\
                  while read -r file; do
                    echo "Generating $file"
                    protoc \
                      -I"${cometbls-src}/proto" \
                      -I"${cosmossdk-src}/proto" \
                      -I"${ibcgo-src}/proto" \
                      -I"${cosmosproto-src}/proto" \
                      -I"${ics23-src}/proto" \
                      -I"${googleapis-src}" \
                      -I"${gogoproto-src}" \
          	          -I"$plugindir/include" \
          	          --plugin="protoc-gen-sol=$plugindir/plugin/gen_sol.py" \
          	          --sol_out=gen_runtime="ProtoBufRuntime.sol&solc_version=0.8.18:$2" \
                      "$file"
                  done
                  find ${../uniond/proto} -type f -regex ".*ibc.*proto" |\
                  while read -r file; do
                    echo "Generating $file"
                    protoc \
                      -I"${cometbls-src}/proto" \
                      -I"${cosmossdk-src}/proto" \
                      -I"${ibcgo-src}/proto" \
                      -I"${cosmosproto-src}/proto" \
                      -I"${ics23-src}/proto" \
                      -I"${googleapis-src}" \
                      -I"${gogoproto-src}" \
                      -I"${../uniond/proto}" \
          	          -I"$plugindir/include" \
          	          --plugin="protoc-gen-sol=$plugindir/plugin/gen_sol.py" \
          	          --sol_out=gen_runtime="ProtoBufRuntime.sol&solc_version=0.8.18:$2" \
                      "$file"
                  done
        '';
      };
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
            yarn
            nodePackages.node-gyp-build
            nodePackages.node-pre-gyp
            (snappy.override { static = true; })
            pkg-config
          ];
          inherit src;
          buildPhase = ''
            export HOME=$(mktemp -d)
            yarn
            patchShebangs .
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
