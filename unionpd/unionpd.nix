{ ... }: {
  perSystem = { self', pkgs, proto, ... }: {
    packages = {
      unionpd = pkgs.buildGoModule ({
        name = "unionpd";
        src = ./.;
        vendorSha256 = null;
        doCheck = true;
      } // (if pkgs.stdenv.isLinux then {
        nativeBuildInputs = [ pkgs.musl ];
        CGO_ENABLED = 0;
        ldflags = [
          "-linkmode external"
          "-extldflags '-static -L${pkgs.musl}/lib'"
        ];
      } else { }));


      unionpd-image = pkgs.dockerTools.buildImage {
        name = "unionpd";

        copyToRoot = pkgs.buildEnv {
          name = "image-root";
          paths = [ pkgs.coreutils-full pkgs.cacert ];
          pathsToLink = [ "/bin" ];
        };
        config = {
          Entrypoint = [ (pkgs.lib.getExe self'.packages.unionpd) ];
          Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
        };
      };

      generate-prover-proto = pkgs.writeShellApplication {
        name = "generate-prover-proto";
        runtimeInputs =
          [ pkgs.protobuf pkgs.protoc-gen-go pkgs.protoc-gen-go-grpc ];
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

      download-circuit =
        let
          files = pkgs.writeText "files.txt" ''
            /vk.bin
            /pk.bin
            /r1cs.bin
          '';
        in
        pkgs.writeShellApplication {
          name = "download-circuit";
          runtimeInputs = [ pkgs.rclone ];
          text = ''
            if [[ "$#" -ne 2 ]]; then
              echo "Invalid arguments, must be: download-circuit [network] [path]"
              exit 1
            fi
            case $1 in
              testnet)
                url="https://testnet.union.cryptware.io"
                ;;
              *)
                echo "Unknown network: $1"
                exit 1
                ;;
            esac
            rclone --progress --no-traverse --http-url "$url" copy :http:/ ./ --files-from=${files}
          '';
        };
    };
  };
}
