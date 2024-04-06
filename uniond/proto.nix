# The protobuf generation process is based on:
#
# - https://github.com/cosmos/cosmos-sdk/blob/bf17fec0e7b83f98be8eba220f1800bd2d7d5011/contrib/devtools/Dockerfile
# - https://github.com/cosmos/cosmos-sdk/blob/bf17fec0e7b83f98be8eba220f1800bd2d7d5011/Makefile#L401
# - https://github.com/cosmos/cosmos-sdk/blob/bf17fec0e7b83f98be8eba220f1800bd2d7d5011/scripts/protocgen.sh
#
{ ... }: {
  perSystem = { pkgs, self', proto, ibc-go, ensureAtRepositoryRoot, mkCi, ... }: {
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

          vendorHash = "sha256-jVOb2uHjPley+K41pV+iMPNx67jtb75Rb/ENhw+ZMoM=";
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

          vendorHash = "sha256-7kDz0RAon2L/3NTHIxya8nWMyN28G9rAfqUu+lbkea4=";
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
        generate-uniond-proto = pkgs.stdenv.mkDerivation {
          name = "generate-uniond-proto";
          pname = "generate-uniond-proto";
          src = ./.;
          buildInputs = [
            pkgs.protobuf
            pkgs.protoc-gen-go
            pkgs.protoc-gen-go-grpc
            pkgs.gnused
            pkgs.gnostic
            pkgs.yq
            pkgs.tree
            cosmos-proto
            gogoproto
            grpc-gateway
          ];

          buildPhase = ''
            mkdir $out
            mkdir $out/openapi

            find ${proto.uniond} -type f -regex ".*proto" | \
            while read -r file; do
              relpath="$(sed 's#/nix/store/.*-uniond/proto##' <<< $file)"
              echo "Generating protobuf for $file"
              mkdir -p "$out/openapi$relpath"
              protoc \
                -I"${proto.uniond}" \
                -I"${proto.gogoproto}" \
                -I"${proto.googleapis}" \
                -I"${proto.cosmossdk}/proto" \
                -I"${proto.cosmosproto}/proto" \
                -I"${proto.ibc-go}/proto" \
                -I"${proto.ics23}/proto" \
                --grpc-gateway_out $out \
                --grpc-gateway_opt=logtostderr=true,allow_colon_final_segments=true \
                --gocosmos_out $out \
                --gocosmos_opt=plugins=interfacetype+grpc,Mgoogle/protobuf/any.proto=github.com/cosmos/cosmos-sdk/codec/types,Mgoogle/protobuf/duration.proto=time \
                --openapi_out=$out/openapi$relpath \
                "$file"
            done

            mkdir -p cosmos-sdk/proto
            cp --no-preserve=mode -RL ${proto.cosmossdk}/proto/* cosmos-sdk/proto

            echo "Generating Cosmos SDK OpenAPI"
            echo "$LINENO"
            find ${proto.cosmossdk}/proto -type f -regex '.*proto' | \
            while read -r file; do
              if grep -q "option go_package" "$file"
              then
                relpath="$(sed 's#/nix/store/.*-source/proto##' <<< $file)"
                prefix="''${relpath%'/'*}"
                maybe_version=''${prefix##*'/'}
                if [[ $maybe_version =~ /(v[0-9]+p[0-9]+(alpha|beta)[0-9]*)|(v[0-9]+(alpha|beta)[0-9]*)|(v[0-9]+)|(v[0-9]+test.*)/g ]]
                then
                  version="''${prefix##*'/'}"
                  del_version_trim="''${prefix%'/'*}"
                  package="''${del_version_trim##*'/'}"
                  sed 's#option go_package.*= ".*";#option go_package = "cosmossdk.io/api'"$prefix;$package$version"'";#' $file > "./cosmos-sdk/proto$relpath"
                else
                  sed 's#option go_package.*= ".*";#option go_package = "cosmossdk.io/api'"$prefix"'";#' $file > "./cosmos-sdk/proto$relpath"
                fi
              else # file is missing `go_package`
                relpath="$(sed 's#/nix/store/.*-source/proto##' <<< $file)"
                prefix="''${relpath%'/'*}"
                maybe_version=''${prefix##*'/'}
                if [[ $maybe_version =~ /(v[0-9]+p[0-9]+(alpha|beta)[0-9]*)|(v[0-9]+(alpha|beta)[0-9]*)|(v[0-9]+)|(v[0-9]+test.*)/g ]]
                then
                  version="''${prefix##*'/'}"
                  del_version_trim="''${prefix%'/'*}"
                  package="''${del_version_trim##*'/'}"
                  sed '2 i option go_package = "cosmossdk.io/api'"$prefix;$package$version"'";' $file > "./cosmos-sdk/proto$relpath"
                else
                  sed '2 i option go_package = "cosmossdk.io/api'"$prefix"'";' $file > "./cosmos-sdk/proto$relpath"
                fi
              fi
            done

            proto_dirs=$(find $(pwd)/cosmos-sdk/proto/cosmos -path -prune -o -name '*.proto' -print0 | xargs -0 -n1 dirname | sort | uniq)
            for dir in $proto_dirs; do
              # generate swagger files (filter query files)
              query_file=$(find "$dir" -maxdepth 1 \( -name 'query.proto' -o -name 'service.proto' \))
              if [[ ! -z "$query_file" ]]; then
                mkdir -p $out/openapi$query_file
                protoc \
                -I"${proto.gogoproto}" \
                -I"${proto.googleapis}" \
                -I"$(pwd)/cosmos-sdk/proto" \
                -I"${proto.cosmosproto}/proto" \
                --openapi_out=$out/openapi$query_file \
                "$query_file"
              fi
            done

            specs=$(find $out/openapi -path -prune -o -name '*.yaml' -print0 | xargs -0 -n1 | sort | uniq)

            yq 'reduce inputs.paths as $s (.; .paths += $s)' ./docs/openapi-base.yaml $specs > openapi_combined.yaml
            yq -s '.[0].paths * .[1].paths | { paths: . }' openapi_combined.yaml ./docs/openapi-overwrites.json > paths.yaml
            yq 'reduce inputs.paths as $s (.; .paths += $s)' openapi_combined.yaml paths.yaml > openapi_combined_overwritten.yaml
            yq 'reduce inputs.components.schemas as $s (.; .components.schemas += $s)' openapi_combined_overwritten.yaml $specs > $out/openapi_combined.yaml

            echo "Patching generated go files to ignore staticcheck warnings"
            find $out -name "*.go" -exec sed -i "1s/^/\/\/lint:file-ignore SA1019 This code is generated\n/" {} +;
          '';
        };
      in
      {
        gen-proto = mkCi false (pkgs.writeShellApplication {
          name = "gen-proto";
          runtimeInputs = (with pkgs; [ buf go gnused ]) ++ [ grpc-gateway cosmos-proto gogoproto ];
          text = ''
            set -eo pipefail

            ${ensureAtRepositoryRoot}

            cd uniond

            echo "Generating go code based on ./uniond/proto"
            echo "Moving patched go sources to correct directories"
            cp --no-preserve=mode -RL ${generate-uniond-proto}/openapi_combined.yaml ./docs/static/openapi.yml
            cp --no-preserve=mode -RL ${generate-uniond-proto}/union/x/* ./x/
            cp --no-preserve=mode -RL ${generate-uniond-proto}/union/staking/* ./x/staking

            echo "Done! Generated .pb.go files are added to ./uniond/x"
          '';
        });
      };
    checks = { };
  };
}
