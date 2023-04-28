# The protobuf generation process is based on:
#
# - https://github.com/cosmos/cosmos-sdk/blob/bf17fec0e7b83f98be8eba220f1800bd2d7d5011/contrib/devtools/Dockerfile
# - https://github.com/cosmos/cosmos-sdk/blob/bf17fec0e7b83f98be8eba220f1800bd2d7d5011/Makefile#L401
# - https://github.com/cosmos/cosmos-sdk/blob/bf17fec0e7b83f98be8eba220f1800bd2d7d5011/scripts/protocgen.sh
#
{ ... }: {
  perSystem = { pkgs, self', ... }: {
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
          pname = "protoc-gen-cosmos";
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
      in
      {
        gen-proto = pkgs.writeShellApplication {
          name = "gen-proto";
          runtimeInputs = (with pkgs; [ buf go ]) ++ [ grpc-gateway cosmos-proto gogoproto ];
          text = ''
            cd uniond
            set -eo pipefail

            echo "Generating go code based on ./uniond/proto"
            cd proto
            buf mod update
            buf generate --template ./buf.gen.gogo.yaml
            cd ..

            # move proto files to the right places
            cp -r ./union/x/* x/
            rm -rf ./union
          '';
        };


      };
    checks = { };
  };
}
