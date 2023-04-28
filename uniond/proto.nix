{ ... }: {
  perSystem = { pkgs, self', ... }: {
    packages = {
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

      gen-proto = pkgs.writeShellApplication {
        name = "gen-proto";
        runtimeInputs = (with pkgs; [ buf go ]) ++ (with self'.packages; [ grpc-gateway cosmos-proto gogoproto ]);
        text = ''
          #== Requirements ==
          ## make sure your `go env GOPATH` is in the `$PATH`
          ## Install:
          ## + latest buf (v1.0.0-rc11 or later)
          ## + protobuf v3
          #
          ## All protoc dependencies must be installed not in the module scope
          ## currently we must use grpc-gateway v1
          # cd ~
          # go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
          # go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest
          # go install github.com/grpc-ecosystem/grpc-gateway/protoc-gen-grpc-gateway@v1.16.0
          # go install github.com/cosmos/cosmos-proto/cmd/protoc-gen-go-pulsar@latest
          # go get github.com/regen-network/cosmos-proto@latest # doesn't work in install mode
          # go get github.com/regen-network/cosmos-proto/protoc-gen-gocosmos@v0.3.1

          set -eo pipefail

          echo "Generating go code based on ./proto"
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
