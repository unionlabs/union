{ ... }: {
  perSystem = { self', pkgs, ... }:
    let
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
    in {
      packages.generate-prover-proto = pkgs.writeShellApplication {
        name = "generate-prover-proto";
        runtimeInputs = [ pkgs.protobuf pkgs.protoc-gen-go pkgs.protoc-gen-go-grpc ];
        text = ''
            find ${../unionpd/proto} -type f -regex ".*proto" |\
            while read -r file; do
              echo "Generating $file"
              protoc \
                -I"${cometbls-src}/proto" \
                -I"${cosmossdk-src}/proto" \
                -I"${ibcgo-src}/proto" \
                -I"${cosmosproto-src}/proto" \
                -I"${ics23-src}/proto" \
                -I"${../unionpd/proto}" \
                -I"${googleapis-src}" \
                -I"${gogoproto-src}" \
                --go_out=./grpc --go_opt=paths=source_relative \
                --go-grpc_out=./grpc --go-grpc_opt=paths=source_relative \
                "$file"
            done
        '';
      };
    };
}
