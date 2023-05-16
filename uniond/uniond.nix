{ ... }: {
  perSystem = { pkgs, self', crane, system, ... }:
    {
      packages = rec {
        wasmvm =
          let
            rustToolchain = pkgs.rust-bin.stable.latest.default.override {
              targets = [ "x86_64-unknown-linux-musl" ];
            };

            craneLib = crane.lib.overrideToolchain rustToolchain;
          in
          craneLib.buildPackage rec {
            src = "${
              pkgs.fetchFromGitHub {
                owner = "CosmWasm";
                repo = "wasmvm";
                rev = "a9e26c0e4e5a076d82556c4f44abeee2a64ff37e";
                hash = "sha256-zR47q8Z2znPigecPDmw5L4ef20/TXv8cPxaXTdJGxg0=";
              }
            }/libwasmvm";

            CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
            # -C target-feature=+crt-static
            # CARGO_BUILD_RUSTFLAGS = "--crate-type=staticlib";
            doCheck = false;
            cargoBuildCommand = "cargo build --release --example=muslc";
            installPhase = ''
              mkdir -p $out/lib
              # mv target/release/libwasmvm.so $out/lib/libwasmvm.$ {
              #   builtins.head (pkgs.lib.strings.splitString "-" system)
              # }.so
              # ls -al target/release
              # ls -al x86_64-unknown-linux-musl
              ls -al target/${CARGO_BUILD_TARGET}/release/examples/libmuslc.a

              mv target/${CARGO_BUILD_TARGET}/release/examples/libmuslc.a $out/lib/libwasmvm.${builtins.head (pkgs.lib.strings.splitString "-" system)}.a
            '';
          };

        uniond = pkgs.pkgsStatic.buildGoModule ({
          name = "uniond";
          src = ./.;
          vendorSha256 = null;
          doCheck = true;
        } // (if pkgs.stdenv.isLinux then {
          # statically link if we're on linux
          nativeBuildInputs = [ pkgs.musl wasmvm ];
          ldflags = [
            "-linkmode external"
            "-extldflags '-static -L${pkgs.musl}/lib -L${wasmvm}/lib'"
          ];
        } else {
          dontFixup = true;
          ldflags = [
            "-extldflags '-L${wasmvm}/lib'"
          ];
        }));

        uniond-image = pkgs.dockerTools.buildImage {
          name = "uniond";

          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            paths = [ pkgs.coreutils-full pkgs.cacert ];
            pathsToLink = [ "/bin" ];
          };
          config = {
            Entrypoint = [ "${self'.packages.uniond}/bin/uniond" ];
            Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
          };
        };
      };

      checks = {
        go-test = pkgs.go.stdenv.mkDerivation {
          name = "go-test";
          buildInputs = [ pkgs.go ];
          src = ./.;
          doCheck = true;
          checkPhase = ''
            # Go will try to create a .cache/ dir in $HOME.
            # We avoid this by setting $HOME to the builder directory
            export HOME=$(pwd)

            go version
            go test ./...
            touch $out
          '';
        };

        go-vet = pkgs.go.stdenv.mkDerivation {
          name = "go-vet";
          buildInputs = [ pkgs.go ];
          src = ./.;
          doCheck = true;
          checkPhase = ''
            # Go will try to create a .cache/ dir in $HOME.
            # We avoid this by setting $HOME to the builder directory
            export HOME=$(pwd)

            go version
            go vet ./...
            touch $out
          '';
        };

        go-staticcheck = pkgs.go.stdenv.mkDerivation {
          name = "go-staticcheck";
          buildInputs = [ pkgs.go pkgs.go-tools ];
          src = ./.;
          doCheck = true;
          checkPhase = ''
            # Go will try to create a .cache/ dir in $HOME.
            # We avoid this by setting $HOME to the builder directory
            export HOME=$(pwd)

            staticcheck ./...
            touch $out
          '';
        };

      };
    };
}
