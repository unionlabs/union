{ self, ... }:
{
  perSystem =
    {
      self',
      pkgs,
      proto,
      ensureAtRepositoryRoot,
      mkCi,
      ...
    }:
    {
      packages = {
        galoisd = pkgs.pkgsStatic.buildGo123Module (
          {
            name = "galoisd";
            src = ./.;
            vendorHash = "sha256-lGqoOkJnTvCdIonLwDDqz9ozDDJwB4wyJXlCgvt4arE=";
            meta = {
              mainProgram = "galoisd";
            };
            tags = [ "binary" ];
            doCheck = true;
          }
          // (
            if pkgs.stdenv.isLinux then
              {
                CGO_ENABLED = 1;
                ldflags = [
                  "-linkmode external"
                  "-extldflags '-z noexecstack -static -L${pkgs.musl}/lib -s -w'"
                ];
              }
            else
              { }
          )
        );

        galoisd-library = pkgs.pkgsStatic.buildGo123Module (
          {
            name = "libgalois";
            src = ./.;
            vendorHash = "sha256-lGqoOkJnTvCdIonLwDDqz9ozDDJwB4wyJXlCgvt4arE=";
            tags = [ "library" ];
            doCheck = false;
          }
          // (
            if pkgs.stdenv.isLinux then
              {
                nativeBuildInputs = [
                  pkgs.musl
                  pkgs.pkgsStatic.binutils
                ];
                doCheck = false;
                CGO_ENABLED = 1;
                GOBIN = "${placeholder "out"}/lib";
                postInstall = ''
                  mv $out/lib/galoisd $out/lib/libgalois.a
                '';
                ldflags = [
                  "-s"
                  "-w"
                  "-buildmode c-archive"
                ];
              }
            else
              { }
          )
        );

        galoisd-image = pkgs.dockerTools.buildImage {
          name = "${self'.packages.galoisd.name}-image";
          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            paths = [
              pkgs.coreutils-full
              pkgs.cacert
            ];
            pathsToLink = [ "/bin" ];
          };
          config = {
            Entrypoint = [ (pkgs.lib.getExe self'.packages.galoisd) ];
            Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
          };
        };

        generate-prover-proto = mkCi false (
          pkgs.writeShellApplication {
            name = "generate-prover-proto";
            runtimeInputs = [
              pkgs.protobuf
              pkgs.protoc-gen-go
              pkgs.protoc-gen-go-grpc
            ];
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
          }
        );

        download-circuit =
          let
            circuit-name = "circuit-eb62b71bc60668da0e602eaa3d6aceec183fb5ca-26eae4b9-bd55-4ce7-8446-ad829ab7b3ed.zip";
            files = pkgs.writeText "files.txt" ''
              /${circuit-name}
            '';
          in
          mkCi false (
            pkgs.writeShellApplication {
              name = "download-circuit";
              runtimeInputs = [
                pkgs.rclone
                pkgs.zip
                pkgs.unzip
              ];
              text = ''
                if [[ "$#" -ne 1 ]]; then
                echo "Invalid arguments, must be: download-circuit [path]"
                exit 1
                fi
                rclone --progress --no-traverse --http-url "https://circuit.cryptware.io" copy :http:/ "$1" --files-from=${files}
                unzip "$1"/${circuit-name}
                rm "$1"/${circuit-name}
              '';
            }
          );

        download-circuit-devnet = pkgs.writeShellApplication {
          name = "download-circuit-devnet";
          runtimeInputs = [
            pkgs.coreutils
            pkgs.zip
            pkgs.unzip
          ];
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

        # Beware this is only valid for the `serve` command of galoisd
        galoisd-testnet-standalone =
          let
            unpackCircuit =
              circuit:
              pkgs.runCommand "galoisd-circuit-${circuit.name}-unpacked" { buildInputs = [ pkgs.unzip ]; } ''
                unzip ${circuit} -d $out
              '';
            unpacked-circuit = unpackCircuit (
              pkgs.fetchurl {
                url = "https://circuit.cryptware.io/circuit-eb62b71bc60668da0e602eaa3d6aceec183fb5ca-26eae4b9-bd55-4ce7-8446-ad829ab7b3ed.zip";
                hash = "sha256-4cExiem1lKrQlDIsrArfQPTuTvpABzi/rNra17R/md4=";
              }
            );
          in
          mkCi false (
            pkgs.symlinkJoin {
              name = "galoisd";
              paths = [ self'.packages.galoisd ];
              buildInputs = [ pkgs.makeWrapper ];
              postBuild = ''
                wrapProgram $out/bin/galoisd \
                  --append-flags "--cs-path ${unpacked-circuit}/r1cs.bin --vk-path ${unpacked-circuit}/vk.bin --pk-path ${unpacked-circuit}/pk.bin" \
                  --set SSL_CERT_FILE "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
              '';
            }
          );

        galoisd-testnet-image = pkgs.dockerTools.buildImage {
          name = "${self'.packages.galoisd.name}-testnet-image";
          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            paths = [
              pkgs.coreutils-full
              pkgs.cacert
            ];
            pathsToLink = [ "/bin" ];
          };
          config = {
            Entrypoint = [ (pkgs.lib.getExe self'.packages.galoisd-testnet-standalone) ];
            Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
          };
        };
      };
    };

  flake.nixosModules.galoisd =
    {
      lib,
      pkgs,
      config,
      ...
    }:
    with lib;
    let
      cfg = config.services.galoisd;
      logLevels = [
        "trace"
        "debug"
        "info"
        "warn"
        "error"
        "fatal"
        "panic"
      ];
    in
    {
      options.services.galoisd = {
        enable = mkEnableOption "Galois daemon service";
        package = mkOption {
          type = types.package;
          default = self.packages.${pkgs.system}.galoisd-testnet-standalone;
        };
        host = mkOption {
          type = types.str;
          default = "localhost:9999";
        };
        max-conn = mkOption {
          type = types.int;
          default = 1;
        };
        log-level = mkOption {
          type = types.enum logLevels;
          default = "info";
        };
      };
      config = mkIf cfg.enable {
        systemd.services.galoisd = {
          wantedBy = [ "multi-user.target" ];
          description = "Galois Daemon";
          serviceConfig = {
            Type = "simple";
            # Default the log-level to `2` because we need to subtract 1 to get the correct value from the enum (index starting at 0).
            # This is because `trace` level starts at -1, https://github.com/rs/zerolog/blob/c78e50e2da70f4ae63e1b65222c3acf12e9ba699/README.md#leveled-logging.
            ExecStart = ''
              ${pkgs.lib.getExe cfg.package} \
                serve ${cfg.host} \
                --max-conn ${builtins.toString cfg.max-conn} \
                --log-level ${
                  builtins.toString ((pkgs.lib.lists.findFirstIndex (x: x == cfg.log-level) 2 logLevels) - 1)
                }
            '';
            Restart = mkForce "always";
            RestartSec = 10;
          };
        };
      };
    };
}
