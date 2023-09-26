{ ... }: {
  perSystem = { self', pkgs, pkgs-unstable, proto, system, ... }: {
    devShells = {
      galois = pkgs.mkShell {
        buildInputs = [
          (pkgs-unstable.cudaPackages_12.cudatoolkit.overrideAttrs (old: old // { meta = null; }))
        ];
      };
    };
    packages =
      let
        cuda = pkgs-unstable.cudaPackages_12.cudatoolkit.overrideAttrs (old: old // { meta = null; });
        cudart = pkgs-unstable.cudaPackages_12.cuda_cudart.overrideAttrs (old: old // { meta = null; });
        icicle =
          let
            isAarch64 = ((builtins.head (pkgs.lib.splitString "-" system)) == "aarch64");
          in
          pkgs.gcc11Stdenv.mkDerivation {
            pname = "icicle";
            version = "0.0.1";
            src = pkgs.fetchFromGitHub {
              owner = "ingonyama-zk";
              repo = "icicle";
              rev = "04e5ff5d1af4";
              hash = "sha256-flqfyD/r614gJPN+w/I+PksJ5gnbltLMXdMq7Vh7ziY=";
            };
            buildPhase = ''
              ${cuda}/bin/nvcc -DG2_DEFINED -Xcompiler -fPIC -std=c++17 -shared -L${cudart}/lib \
                  icicle/curves/bn254/lde.cu \
                  icicle/curves/bn254/msm.cu \
                  icicle/curves/bn254/projective.cu \
                  icicle/curves/bn254/ve_mod_mult.cu \
                  -o libbn254.so
            '';
            installPhase = ''
              mkdir -p $out/lib
              mv libbn254.so $out/lib
            '';
            enableParallelBuilding = true;
            doCheck = true;
          };
        galoisd = pkgs.buildGoModule ({
          name = "galoisd";
          src = ./.;
          vendorSha256 = null;
          doCheck = false;
          meta = {
            mainProgram = "galoisd";
          };
          tags = [ "!gpu" ];
        } // (if pkgs.stdenv.isLinux then {
          nativeBuildInputs = [ pkgs.musl ];
          CGO_ENABLED = 0;
          ldflags = [
            "-linkmode external"
            "-extldflags '-static -L${pkgs.musl}/lib -s -w'"
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

        mkGaloisdGpu = { network, maxVal }:
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
            CGO_ENABLED = 1;
            CGO_CFLAGS = "-I${cuda}/include -I${icicle.src}/goicicle/curves/bn254/include";
            CC = "${pkgs.gcc11}/bin/gcc";
            ldflags = [
              "-extldflags '-L${icicle}/lib -L${cudart}/lib -s -w'"
            ];
            tags = [ "gpu" ];
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

        galoisd-devnet-gpu = mkGaloisdGpu { network = "devnet"; maxVal = 4; };
        galoisd-testnet-gpu = mkGaloisdGpu { network = "testnet"; maxVal = 16; };
        galoisd-mainnet-gpu = mkGaloisdGpu { network = "mainnet"; maxVal = 128; };

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
