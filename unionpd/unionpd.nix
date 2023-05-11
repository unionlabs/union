{ ... }: {
  perSystem = { self', pkgs, proto, ... }:
    {
      packages.generate-prover-proto = pkgs.writeShellApplication {
        name = "generate-prover-proto";
        runtimeInputs = [ pkgs.protobuf pkgs.protoc-gen-go pkgs.protoc-gen-go-grpc ];
        text = ''
          find ${proto.unionpd} -type f -regex ".*proto" |\
          while read -r file; do
            echo "Generating $file"
            protoc \
               -I"${proto.cometbls}/proto" \
               -I"${proto.gogoproto}" \
               -I"${proto.unionpd}" \
              --go_out=./grpc --go_opt=paths=source_relative \
              --go-grpc_out=./grpc --go-grpc_opt=paths=source_relative \
              "$file"
          done
        '';
      };
    };
}
