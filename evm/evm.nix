{ ... }: {
  perSystem = { self', pkgs, proto, ... }: {
    packages.generate-evm-proto = pkgs.writeShellApplication {
      name = "generate-evm-proto";
      runtimeInputs = [ pkgs.protobuf ];
      text = let
        solidity-protobuf = pkgs.stdenv.mkDerivation {
          name = "solidity-protobuf";
          version = "0.0.1";
          src = pkgs.fetchFromGitHub {
            owner = "CyrusVorwald";
            repo = "solidity-protobuf";
            rev = "1c323bed92d373d6c4d6c728c8dd9f76cf4b5a0c";
            hash = "sha256-1obEhMjaLToaSk920CiJwfhkw+LDgY5Y/b7SpkeuqDE=";
          };
          buildInputs =
            [ (pkgs.python3.withPackages (ps: with ps; [ protobuf wrapt ])) ];
          buildPhase = "true";
          installPhase = ''
            mkdir $out
            cp -r $src/* $out
          '';
        };
        protoIncludes = ''-I"${proto.cometbls}/proto" -I"${proto.cosmossdk}/proto" -I"${proto.ibcgo}/proto" -I"${proto.cosmosproto}/proto" -I"${proto.ics23}/proto" -I"${proto.googleapis}" -I"${proto.gogoproto}" -I"${proto.uniond}"'';
      in ''
                plugindir="${solidity-protobuf}/protobuf-solidity/src/protoc"
                find ${proto.ibcgo}/proto -name "$1" |\
                while read -r file; do
                  echo "Generating $file"
                  protoc \
                    ${protoIncludes} \
        	          -I"$plugindir/include" \
        	          --plugin="protoc-gen-sol=$plugindir/plugin/gen_sol.py" \
        	          --sol_out=gen_runtime="ProtoBufRuntime.sol&solc_version=0.8.18:$2" \
                    "$file"
                done
                find ${proto.cometbls}/proto -type f -regex ".*canonical.proto" |\
                while read -r file; do
                  echo "Generating $file"
                  protoc \
                    ${protoIncludes} \
        	          -I"$plugindir/include" \
        	          --plugin="protoc-gen-sol=$plugindir/plugin/gen_sol.py" \
        	          --sol_out=gen_runtime="ProtoBufRuntime.sol&solc_version=0.8.18:$2" \
                    "$file"
                done
                find ${proto.uniond} -type f -regex ".*ibc.*cometbls.*proto" |\
                while read -r file; do
                  echo "Generating $file"
                  protoc \
                    ${protoIncludes} \
        	          -I"$plugindir/include" \
        	          --plugin="protoc-gen-sol=$plugindir/plugin/gen_sol.py" \
        	          --sol_out=gen_runtime="ProtoBufRuntime.sol&solc_version=0.8.18:$2" \
                    "$file"
                done
      '';
    };
  };
}
