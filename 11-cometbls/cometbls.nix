{ ... }: {
  perSystem = { self', pkgs, proto, goPkgs, ensureAtRepositoryRoot, mkCi, ... }: {
    packages = let
      cosmos-proto = pkgs.buildGoModule {
        pname = "cosmos-proto";
        version = "1.0.0";
        src = pkgs.fetchFromGitHub {
          owner = "cosmos";
          repo = "cosmos-proto";
          rev = "v1.0.0-beta.3";
          sha256 = "sha256-kFm1ChSmm5pU9oJqKmWq4KfO/hxgxzvcSzr66oTulos=";
        };
        doCheck = false;

        vendorHash = "sha256-7kDz0RAon2L/3NTHIxya8nWMyN28G9rAfqUu+lbkea4=";
      };
      grpc-gateway = pkgs.buildGoModule {
        pname = "grpc-gateway";
        version = "1.16.0";
        src = pkgs.fetchFromGitHub {
          owner = "grpc-ecosystem";
          repo = "grpc-gateway";
          rev = "v1.16.0";
          sha256 = "sha256-jJWqkMEBAJq50KaXccVpmgx/hwTdKgTtNkz8/xYO+Dc=";
        };

        vendorHash = "sha256-jVOb2uHjPley+K41pV+iMPNx67jtb75Rb/ENhw+ZMoM=";
      };
      gogoproto = pkgs.buildGoModule {
        pname = "gogoproto";
        version = "1.4.7";
        src = pkgs.fetchFromGitHub {
          owner = "cosmos";
          repo = "gogoproto";
          rev = "v1.4.7";
          sha256 = "sha256-oaGwDFbz/xgL7hDtvdh/mIcRIGBdp+/xuKeuBE2ZpqY=";
        };
        nativeBuildInputs = with pkgs; [ protobuf ];
        doCheck = false;

        vendorHash = "sha256-nfeqVsPMQz7EL+qWxFzRukCE3YqXErhS9urRaJo44Fg=";
      };
      
      generate-cometbls-lc-proto = pkgs.stdenv.mkDerivation {
        name = "generate-cometbls-lc-proto";
        pname = "generate-cometbls-lc-proto";
        src = ./.;
        buildInputs = [ gogoproto grpc-gateway cosmos-proto pkgs.protobuf pkgs.protoc-gen-go pkgs.protoc-gen-go-grpc ];

        buildPhase = ''
          mkdir $out
          
          find ${proto.cometbls-lc} -type f -regex ".*proto" |\
          while read -r file; do
          echo "Generating $file"
          protoc \
          -I"${proto.cometbls}/proto" \
          -I"${proto.gogoproto}" \
          -I"${proto.googleapis}" \
          -I"${proto.cometbls-lc}" \
          -I"${proto.cosmosproto}/proto" \
          -I"${proto.cosmossdk}/proto" \
          --go_out $out \
          --go_opt=paths=source_relative \
          --go-grpc_out $out \
          --go-grpc_opt=paths=source_relative \
          --grpc-gateway_out $out \
          --grpc-gateway_opt=logtostderr=true,allow_colon_final_segments=true \
          --gocosmos_out $out \
          --gocosmos_opt=plugins=interfacetype+grpc,Mgoogle/protobuf/any.proto=github.com/cosmos/cosmos-sdk/codec/types,Mgoogle/protobuf/duration.proto=time \
          "$file"
          done
        '';
      };
    in {
      generate-cometbls-lc-proto = mkCi false (pkgs.writeShellApplication {
        name = "generate-cometbls-proto";
        text = ''
          set -eo pipefail

          ${ensureAtRepositoryRoot}

          cd 11-cometbls

          echo "Generating go code based on ./uniond/proto"
          echo "Moving patched go sources to correct directories"
          cp --no-preserve=mode -RL ${generate-cometbls-lc-proto}/github.com/unionlabs/union/11-cometbls/types/* ./types/

          echo "Done! Generated .pb.go files are added to ./uniond/x"
        '';
      });
    };
  };
}
