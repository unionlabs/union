{ ... }: {
  perSystem = { self', pkgs, proto, goPkgs, ensureAtRepositoryRoot, mkCi, ... }: {
    packages = {
      galoisd-coverage-show =
        pkgs.writeShellApplication {
          name = "galoisd-coverage-show";
          runtimeInputs = [ goPkgs.go ];
          text = ''
            ${ensureAtRepositoryRoot}
            pushd galoisd
            go tool cover -html=${self'.packages.galoisd-coverage}
            popd
          '';
        };

      galoisd-coverage =
        pkgs.runCommand
          "galoisd-coverage"
          { buildInputs = [ goPkgs.go ]; }
          ''
            HOME="$(mktemp -d)"
            cd ${./.}
            go test -v -coverpkg=./... -coverprofile=$out ./...
          '';

      galoisd = goPkgs.buildGoModule ({
        name = "galoisd";
        src = ./.;
        vendorHash = null;
        meta = { mainProgram = "galoisd"; };
      } // (if pkgs.stdenv.isLinux then {
        nativeBuildInputs = [ pkgs.musl ];
        CGO_ENABLED = 0;
        ldflags = [
          "-linkmode external"
          "-extldflags '-static -L${pkgs.musl}/lib -s -w'"
        ];
      } else
        { }));

      galoisd-image = pkgs.dockerTools.buildImage {
        name = "${self'.packages.galoisd.name}-image";
        copyToRoot = pkgs.buildEnv {
          name = "image-root";
          paths = [ pkgs.coreutils-full pkgs.cacert ];
          pathsToLink = [ "/bin" ];
        };
        config = {
          Entrypoint = [ (pkgs.lib.getExe self'.packages.galoisd) ];
          Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
        };
      };

      generate-prover-proto = mkCi false (pkgs.writeShellApplication {
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
      });

      download-circuit =
        let
          files = pkgs.writeText "files.txt" ''
            /circuit.zip
          '';
        in
        mkCi false (pkgs.writeShellApplication {
          name = "download-circuit";
          runtimeInputs = [ pkgs.rclone pkgs.zip pkgs.unzip ];
          text = ''
            if [[ "$#" -ne 1 ]]; then
            echo "Invalid arguments, must be: download-circuit [path]"
            exit 1
            fi
            rclone --progress --no-traverse --http-url "https://circuit.cryptware.io" copy :http:/ "$1" --files-from=${files}
            unzip "$1"/circuit.zip
            rm "$1"/circuit.zip
          '';
        });

      download-circuit-devnet =
        pkgs.writeShellApplication {
          name = "download-circuit-devnet";
          runtimeInputs = [ pkgs.coreutils pkgs.zip pkgs.unzip ];
          text = ''
            ${ensureAtRepositoryRoot}

            if [[ -f "./.devnet/circuit/r1cs.bin" && -f "./.devnet/circuit/pk.bin" && -f "./.devnet/circuit/vk.bin" ]] 
            then
              echo "Circuit is already downloaded"
              exit 0
            fi

            mkdir -p .devnet/circuit/
            cd .devnet/circuit/

            echo "Downloading circuit"
            ${pkgs.lib.getExe self'.packages.download-circuit} .
          '';
        };
    };
  };
}
