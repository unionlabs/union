{ ... }: {
  perSystem = { self', pkgs, proto, ... }: {
    packages =
      let
        galoisd = pkgs.buildGoModule ({
          name = "galoisd";
          src = ./.;
          vendorSha256 = null;
          doCheck = false;
          meta = {
            mainProgram = "galoisd";
          };
        } // (if pkgs.stdenv.isLinux then {
          nativeBuildInputs = [ pkgs.musl ];
          CGO_ENABLED = 0;
          ldflags = [
            "-linkmode external"
            "-extldflags '-static -L${pkgs.musl}/lib'"
          ];
        } else { }));

        mkGaloisd = { network, maxVal }:
          galoisd.overrideAttrs (old: {
            name = old.name + "-${network}";
            src = pkgs.runCommand "src-patched" { } ''
              mkdir -p $out
              cp -r ${old.src}/* $out/
              substituteInPlace $out/pkg/lightclient/common.go \
              --replace "const MaxVal = 16" "const MaxVal = ${
                builtins.toString maxVal
              }"
            '';
          });

        mkGaloisdImage = { galoisd }:
          pkgs.dockerTools.buildImage {
            name = "${galoisd.name}-image";
            copyToRoot = pkgs.buildEnv {
              name = "image-root";
              paths = [ pkgs.coreutils-full pkgs.cacert ];
              pathsToLink = [ "/bin" ];
            };
            config = {
              Entrypoint = [ (pkgs.lib.getExe galoisd) ];
              Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
            };
          };
      in
      {
        galoisd-devnet = mkGaloisd { network = "devnet"; maxVal = 4; };
        galoisd-testnet = mkGaloisd { network = "testnet"; maxVal = 16; };
        galoisd-mainnet = mkGaloisd { network = "mainnet"; maxVal = 128; };

        galoisd-devnet-image = mkGaloisdImage { galoisd = self'.packages.galoisd-devnet; };
        galoisd-testnet-image = mkGaloisdImage { galoisd = self'.packages.galoisd-testnet; };
        galoisd-mainnet-image = mkGaloisdImage { galoisd = self'.packages.galoisd-mainnet; };

        generate-prover-proto = pkgs.writeShellApplication {
          name = "generate-prover-proto";
          runtimeInputs =
            [ pkgs.protobuf pkgs.protoc-gen-go pkgs.protoc-gen-go-grpc ];
          text = ''
            find ${proto.galoisd} -type f -regex ".*proto" |\
            while read -r file; do
              echo "Generating $file"
              protoc \
                 -I"${proto.cometbls}/proto" \
                 -I"${proto.gogoproto}" \
                 -I"${proto.galoisd}" \
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
                echo "Invalid arguments, must be: download-circuit (devnet|testnet|mainnet) [path]"
                exit 1
              fi
              case $1 in
                devnet)
                  url="https://devnet.union.cryptware.io"
                  ;;
                testnet)
                  url="https://testnet.union.cryptware.io"
                  ;;
                mainnet)
                  echo "Mainnet circuit has not been uploaded yet"
                  exit 1
                  ;;
                *)
                  echo "Unknown network: $1, must be one of devnet|testnet|mainnet"
                  exit 1
                  ;;
              esac
              rclone --progress --no-traverse --http-url "$url" copy :http:/ "$2" --files-from=${files}
            '';
          };
      };
  };
}
