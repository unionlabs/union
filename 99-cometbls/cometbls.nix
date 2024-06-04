{ ... }: {
  perSystem = { self', pkgs, proto, goPkgs, ensureAtRepositoryRoot, mkCi, ... }: {
    packages = {
      generate-cometbls-lc-proto = mkCi false (pkgs.writeShellApplication {
        name = "generate-cometbls-lc-proto";
        runtimeInputs = [ pkgs.protobuf pkgs.protoc-gen-go ];
        text = ''
          find ${proto.cometbls-lc} -type f -regex ".*proto" |\
          while read -r file; do
            echo "Generating $file"
            protoc \
              -I"${proto.gogoproto}" \
              -I"${proto.cometbls-lc}" \
              --go_out=./99-cometbls/types --go_opt=paths=source_relative \
              "$file"
          done
        '';
      });     
    };
  };
}
  
