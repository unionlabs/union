{ ... }: {
  perSystem = { self', pkgs, proto, goPkgs, ensureAtRepositoryRoot, mkCi, ... }: {
    packages = {
      generate-cometbls-lc-proto = mkCi false (pkgs.writeShellApplication {
        name = "generate-cometbls-proto";
        runtimeInputs =
          [ pkgs.protobuf pkgs.protoc-gen-go pkgs.protoc-gen-go-grpc ];
        text = ''
          find ${proto.cometbls-lc} -type f -regex ".*proto" |\
          while read -r file; do
          echo "Generating $file"
          protoc \
          -I"${proto.cometbls}/proto" \
          -I"${proto.gogoproto}" \
          -I"${proto.cometbls-lc}" \
          -I"${proto.cosmosproto}/proto" \
          -I"${proto.cosmossdk}/proto" \
          --go_out=./types --go_opt=paths=source_relative \
          --go-grpc_out=./types --go-grpc_opt=paths=source_relative \
          "$file"
          done
        '';
      });
    };
  };
}
