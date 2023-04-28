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

      protoc-gen-cosmos = pkgs.buildGoModule {
        pname = "protoc-gen-cosmos";
        version = "1.16.0";
        src = pkgs.fetchFromGitHub {
          owner = "regen-network";
          repo = "cosmos-proto";
          rev = "v0.3.1";
          sha256 = "sha256-Bchbq/Hg72EA7Hevs8+PNuENuQaZAzk3qeVjMqFMUxc=";
        };

        vendorSha256 = "sha256-d3qVcgL0Lil2jSNfgC9hPPNDidSzITUAoJRiHQrExrw=";
      };

      gen-proto = pkgs.writeShellApplication {
        name = "gen-proto";
        runtimeInputs = [ pkgs.buf pkgs.go pkgs.protobuf self'.packages.grpc-gateway self'.packages.protoc-gen-cosmos ];
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
          cd ..
          buf generate

          # move proto files to the right places
          # cp -r ./union/x/* x/
          # rm -rf ./union
        '';

      };


    };
    checks = { };
  };
}
