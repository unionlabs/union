# The protobuf generation process is based on:
#
# - https://github.com/cosmos/cosmos-sdk/blob/bf17fec0e7b83f98be8eba220f1800bd2d7d5011/contrib/devtools/Dockerfile
# - https://github.com/cosmos/cosmos-sdk/blob/bf17fec0e7b83f98be8eba220f1800bd2d7d5011/Makefile#L401
# - https://github.com/cosmos/cosmos-sdk/blob/bf17fec0e7b83f98be8eba220f1800bd2d7d5011/scripts/protocgen.sh
#
{ ... }: {
  perSystem = { pkgs, self', proto, ibc-go, ensureAtRepositoryRoot, ... }: {
    packages =
      let
        grpc-gateway = pkgs.buildGoModule {
          pname = "grpc-gateway";
          version = "1.16.0";
          src = pkgs.fetchFromGitHub {
            owner = "grpc-ecosystem";
            repo = "grpc-gateway";
            rev = "v1.16.0";
            sha256 = "sha256-jJWqkMEBAJq50KaXccVpmgx/hwTdKgTtNkz8/xYO+Dc=";
          };

          vendorSha256 = "sha256-jVOb2uHjPley+K41pV+iMPNx67jtb75Rb/ENhw+ZMoM=";
        };

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

          vendorSha256 = "sha256-7kDz0RAon2L/3NTHIxya8nWMyN28G9rAfqUu+lbkea4=";
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

          vendorSha256 = "sha256-nfeqVsPMQz7EL+qWxFzRukCE3YqXErhS9urRaJo44Fg=";
        };
        generate-uniond-proto = pkgs.stdenv.mkDerivation {
          name = "generate-uniond-proto";
          pname = "generate-uniond-proto";
          src = ./.;
          buildInputs = [
            pkgs.protobuf
            pkgs.protoc-gen-go
            pkgs.protoc-gen-go-grpc
            pkgs.gnused
            cosmos-proto
            gogoproto
            grpc-gateway
          ];

          buildPhase = ''
            mkdir $out

            find ${proto.uniond} -type f -regex ".*proto" | \
            while read -r file; do
              echo "Generating protobuf for $file"
              protoc \
                -I"${proto.uniond}" \
                -I"${proto.gogoproto}" \
                -I"${proto.googleapis}" \
                -I"${proto.cosmossdk}/proto" \
                -I"${proto.cosmosproto}/proto" \
                -I"${proto.ibcgo}/proto" \
                -I"${proto.ics23}/proto" \
                --grpc-gateway_out $out \
                --grpc-gateway_opt=logtostderr=true,allow_colon_final_segments=true \
                --gocosmos_out $out \
                --gocosmos_opt=plugins=interfacetype+grpc,Mgoogle/protobuf/any.proto=github.com/cosmos/cosmos-sdk/codec/types,Mgoogle/protobuf/duration.proto=time \
                "$file"
            done

            echo "Patching generated go files to ignore staticcheck warnings"
            find $out -name "*.go" -exec sed -i "1s/^/\/\/lint:file-ignore SA1019 This code is generated\n/" {} +;
          '';
        };
      in
      {
        gen-proto = pkgs.writeShellApplication {
          name = "gen-proto";
          runtimeInputs = (with pkgs; [ buf go gnused ]) ++ [ grpc-gateway cosmos-proto gogoproto ];
          text = ''
            set -eo pipefail

            ${ensureAtRepositoryRoot}

            cd uniond

            echo "Generating go code based on ./uniond/proto"
            echo "Moving patched go sources to correct directories"
            cp -r ${generate-uniond-proto}/union/x/* ./x/

            cp ${generate-uniond-proto}/union/app/ibc/cometbls/02-client/keeper/* ./app/ibc/cometbls/02-client/keeper/

            echo "Done! Generated .pb.go files are added to ./uniond/x"
          '';
        };
      };
    checks = { };
  };
}
