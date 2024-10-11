{ ... }:
{
  perSystem = { pkgs, self', inputs', proto, ... }: {
    packages =
      let
        cosmos-proto = pkgs.buildGoModule {
          pname = "cosmos-proto";
          version = "1.0.0";
          src = pkgs.fetchFromGitHub {
            owner = "cosmos";
            repo = "cosmos-proto";
            rev = "0748a2ad4a5c78b1db6c8090db01e255bcc91365";
            sha256 = "sha256-vMq1+gMLgUdhClbz38+U3MIh9Wzw0DvyAMX6M4ZZBso=";
          };
          doCheck = false;

          vendorHash = "sha256-Jq0iDd2E4guNRVeUei7eM0wkg6pKm76DY7St5FE//3Q=";
        };

        gogoproto = pkgs.buildGoModule {
          pname = "gogoproto";
          version = "1.4.7";
          src = pkgs.fetchFromGitHub {
            owner = "cosmos";
            repo = "gogoproto";
            rev = "34f37065b54523d08d7b637c78333d444f350e21";
            sha256 = "sha256-jRC3CImRLzReOcNR483jYPNXhcX4eY9seLJe5DOjp7o=";
          };
          nativeBuildInputs = with pkgs; [ protobuf ];
          doCheck = false;

          vendorHash = "sha256-QQNw5NTaAZ7vrgxOoLsuqzwqoWEyYQ+IdFipfmM+PZo=";
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

        protoc-gen-go-cosmos-orm = pkgs.buildGoModule {
          pname = "protoc-gen-go-cosmos-orm";
          version = "1.0.0-beta.3";
          src = "${proto.protoc-gen-go-cosmos-orm}/orm";
          vendorHash = "sha256-7Ww/drdom370PIXC60g/O9BnxpY3u0y7JsrLGJy7CqQ=";
          doCheck = false;
        };

        proto-generate = pkgs.stdenv.mkDerivation {
          name = "generate-proto";
          src = proto.cosmossdk;
          buildInputs = (with pkgs; [
            protobuf
            protoc-gen-go
            protoc-gen-go-grpc
          ]) ++ [
            cosmos-proto
            gogoproto
            grpc-gateway
          ];

          installPhase = ''
            touch $out
          '';
          buildPhase = ''
            mkdir $out
            
            echo "Generating gogo proto code..."
            echo "- Generating ./proto/{ cosmos, amino }"
            proto_dirs=$(find ${proto.cosmossdk}/proto/cosmos ${proto.cosmossdk}/proto/amino -path -prune -o -name '*.proto' -print0 | xargs -0 -n1 dirname | sort | uniq)
            for dir in $proto_dirs; do
              for file in $(find "$dir" -maxdepth 1 -name '*.proto'); do
                # this regex checks if a proto file has its go_package set to cosmossdk.io/api/...
                # gogo proto files SHOULD ONLY be generated if this is false
                # we don't want gogo proto to run for proto files which are natively built for google.golang.org/protobuf
                if grep -q "option go_package" "$file" && ! grep -q 'option go_package.*cosmossdk.io/api' "$file"; then
                  echo "  - Generating protobuf for $file"
                  protoc \
                    -I"${proto.cosmossdk}/proto/" \
                    -I"${proto.gogoproto}" \
                    -I"${proto.googleapis}" \
                    -I"${proto.cosmosproto}/proto" \
                    -I"${proto.cometbft}/proto" \
                    --gocosmos_out $out \
                    --gocosmos_opt=plugins=grpc,Mgoogle/protobuf/any.proto=github.com/cosmos/cosmos-sdk/codec/types \
                    --grpc-gateway_out $out \
                    --grpc-gateway_opt=logtostderr=true,allow_colon_final_segments=true \
                    $file
                fi
              done
            done

            # generate codec/testdata proto code
            echo "- Generating ./testutil/testdata"
            find ${proto.cosmossdk}/testutil/testdata -type f -regex '.*proto' | \
            while read -r file; do
              echo "  - Generating protobuf for $file"
              protoc \
                -I"${proto.cosmossdk}/testutil/testdata" \
                -I"${proto.cosmossdk}/proto" \
                -I"${proto.gogoproto}" \
                -I"${proto.googleapis}" \
                -I"${proto.cosmosproto}/proto" \
                -I"${proto.cometbft}/proto" \
                --gocosmos_out $out \
                --gocosmos_opt=plugins=grpc,Mgoogle/protobuf/any.proto=github.com/cosmos/cosmos-sdk/codec/types \
                $file
            done

            # generate baseapp test messages
            echo "- Generating ./baseapp/testutil"
            find ${proto.cosmossdk}/baseapp/testutil -type f -regex '.*proto' | \
            while read -r file; do
              echo "  - Generating protobuf for $file"
              protoc \
                -I"${proto.cosmossdk}/baseapp/testutil" \
                -I"${proto.cosmossdk}/proto" \
                -I"${proto.gogoproto}" \
                -I"${proto.googleapis}" \
                -I"${proto.cosmosproto}" \
                -I"${proto.cometbft}/proto" \
                --gocosmos_out $out \
                --gocosmos_opt=plugins=grpc,Mgoogle/protobuf/any.proto=github.com/cosmos/cosmos-sdk/codec/types \
                $file
            done

            # move proto files to the right places
            cp -r $out/github.com/cosmos/cosmos-sdk/* $out
            rm -rf $out/github.com

            echo "Generating pulsar proto code..."

            # go_package_prefix:
            #   default: cosmossdk.io/api
            #   except:
            #     - buf.build/googleapis/googleapis
            #     - buf.build/cosmos/gogo-proto
            #     - buf.build/cosmos/cosmos-proto
            echo "- Generating API module"
            find ${proto.cosmossdk}/proto -type f -regex '.*proto' | \
            while read -r file; do
              if grep -q "option go_package" "$file"
              then
                relpath="$(sed 's#/nix/store/.*-cosmos-sdk/proto##' <<< $file)"
                prefix="''${relpath%'/'*}"
                maybe_version=''${prefix##*'/'}
                if [[ $maybe_version =~ /(v[0-9]+p[0-9]+(alpha|beta)[0-9]*)|(v[0-9]+(alpha|beta)[0-9]*)|(v[0-9]+)|(v[0-9]+test.*)/g ]]
                then
                  version="''${prefix##*'/'}"
                  del_version_trim="''${prefix%'/'*}"
                  package="''${del_version_trim##*'/'}"
                  sed 's#option go_package.*= ".*";#option go_package = "cosmossdk.io/api'"$prefix;$package$version"'";#' $file > "./proto$relpath"
                else
                  sed 's#option go_package.*= ".*";#option go_package = "cosmossdk.io/api'"$prefix"'";#' $file > "./proto$relpath"
                fi
              else # file is missing `go_package`
                relpath="$(sed 's#/nix/store/.*-cosmos-sdk/proto##' <<< $file)"
                prefix="''${relpath%'/'*}"
                maybe_version=''${prefix##*'/'}
                if [[ $maybe_version =~ /(v[0-9]+p[0-9]+(alpha|beta)[0-9]*)|(v[0-9]+(alpha|beta)[0-9]*)|(v[0-9]+)|(v[0-9]+test.*)/g ]]
                then
                  version="''${prefix##*'/'}"
                  del_version_trim="''${prefix%'/'*}"
                  package="''${del_version_trim##*'/'}"
                  sed '2 i option go_package = "cosmossdk.io/api'"$prefix;$package$version"'";' $file > "./proto$relpath"
                else
                  sed '2 i option go_package = "cosmossdk.io/api'"$prefix"'";' $file > "./proto$relpath"
                fi
              fi
            done
            mkdir $out/api
            find /build/cosmos-sdk/proto -type f -regex '.*proto' | \
            while read -r file; do
              echo "  - Generating protobuf for $file"
              if grep -q "option go_package" "$file"
              then
                protoc \
                  -I"${proto.gogoproto}" \
                  -I"${proto.googleapis}" \
                  -I"${proto.cosmosproto}/proto" \
                  -I"${proto.cometbft}/proto" \
                  -I"/build/cosmos-sdk/proto" \
                  --go-pulsar_out $out/api \
                  --go-pulsar_opt=paths=source_relative \
                  --go-grpc_out $out/api \
                  --go-grpc_opt=paths=source_relative \
                  $file
              fi
            done

            # go_package_prefix:
            #   default: github.com/cosmos/cosmos-sdk/testutil/testdata_pulsar
            #   except:
            #     - buf.build/googleapis/googleapis
            #     - buf.build/cosmos/gogo-proto
            #     - buf.build/cosmos/cosmos-proto
            #   override:
            #     buf.build/cosmos/cosmos-sdk: cosmossdk.io/api
            echo "- Generating Test Data"
            find ${proto.cosmossdk}/testutil -type f -regex '.*proto' | \
            while read -r file; do
              if grep -q "option go_package" "$file"
              then
                relpath="$(sed 's#/nix/store/.*-cosmos-sdk/testutil##' <<< $file)"
                sed 's#option go_package.*= ".*";#option go_package = "github.com/cosmos/cosmos-sdk/testutil/testpb";#' $file > "./testutil$relpath"
              fi
            done
            find ${proto.cosmossdk}/proto -type f -regex '.*proto' | \
            while read -r file; do
              if grep -q "option go_package" "$file"
              then
                relpath="$(sed 's#/nix/store/.*-cosmos-sdk/proto##' <<< $file)"
                prefix="''${relpath%'/'*}"
                maybe_version=''${prefix##*'/'}
                if [[ $maybe_version =~ /(v[0-9]+p[0-9]+(alpha|beta)[0-9]*)|(v[0-9]+(alpha|beta)[0-9]*)|(v[0-9]+)|(v[0-9]+test.*)/g ]]
                then
                  version="''${prefix##*'/'}"
                  del_version_trim="''${prefix%'/'*}"
                  package="''${del_version_trim##*'/'}"
                  sed 's#option go_package.*= ".*";#option go_package = "cosmossdk.io/api'"$prefix;$package$version"'";#' $file > "./proto$relpath"
                else
                  sed 's#option go_package.*= ".*";#option go_package = "cosmossdk.io/api'"$prefix"'";#' $file > "./proto$relpath"
                fi
              fi
            done
            find ${proto.cosmossdk}/testutil/testdata -type f -regex '.*proto' | \
            while read -r file; do
              echo "  - Generating protobuf for $file"
              relpath="$(sed 's#/nix/store/.*-cosmos-sdk/testutil##' <<< $file)"
              echo "File path: $(readlink -f ./testutil$relpath)"
              protoc \
                -I"/build/cosmos-sdk/testutil/testdata" \
                -I"${proto.gogoproto}" \
                -I"/build/cosmos-sdk/proto" \
                -I"${proto.cosmosproto}/proto" \
                -I"${proto.cometbft}/proto" \
                --go-pulsar_out $out/testutil/testdata \
                --go-pulsar_opt=paths=source_relative \
                --go-grpc_out $out/testutil/testdata \
                --go-grpc_opt=paths=source_relative \
                $(readlink -f ./testutil$relpath)
            done
          '';
        };
      in
      {
        proto-gen = pkgs.writeShellApplication {
          name = "gen-proto";
          runtimeInputs = with pkgs; [ go ];
          text = ''
            set -eo pipefail

            # If the current directory contains flake.nix, then we are at the repository root
            if [[ -f flake.nix ]]
            then
              echo "We are at the repository root. Starting generation..."
            else
              echo "We are NOT at the repository root. Please cd to the repository root and try again."
              exit 1
            fi

            echo "Generating go code based on ./proto"
            echo "Moving patched go sources to correct directories"
            cp -r --no-preserve=mode,ownership ${proto-generate}/* ./.

            echo "Done!"

          '';
        };
        proto-gen-buf = pkgs.writeShellApplication {
          name = "proto-gen-buf";
          runtimeInputs = (with pkgs; [ 
            go_1_23 
            buf 
            clang-tools 
            protobuf
            protoc-gen-go
            protoc-gen-go-grpc
          ] ++ [ 
            cosmos-proto 
            grpc-gateway 
            gogoproto 
            protoc-gen-go-cosmos-orm
          ]);
          text = ''
            # If the current directory contains flake.nix, then we are at the repository root
            if [[ -f flake.nix ]]
            then
              echo "We are at the repository root. Starting generation..."
            else
              echo "We are NOT at the repository root. Please cd to the repository root and try again."
              exit 1
            fi

            ./scripts/protocgen.sh
          '';
        };
      };
  };
}
