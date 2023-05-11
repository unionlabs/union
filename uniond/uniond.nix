{ ... }: {
  perSystem = { pkgs, self', system, crane, ... }: 
    let 
      wasmvm = crane.lib.buildPackage {
        src = "${
            pkgs.fetchFromGitHub {
              owner = "CosmWasm";
              repo = "wasmvm";
              rev = "a9e26c0e4e5a076d82556c4f44abeee2a64ff37e";
              hash = "sha256-zR47q8Z2znPigecPDmw5L4ef20/TXv8cPxaXTdJGxg0=";
            }
          }/libwasmvm";
        doCheck = false;
        installPhase = ''
          mkdir -p $out/lib
          mv target/release/libwasmvm.so $out/lib/libwasmvm.${
            builtins.head (pkgs.lib.strings.splitString "-" system)
          }.so
        '';
      };
    in
  {
    packages = {
      uniond = pkgs.buildGoModule {
        name = "uniond";
        src = ./.;
        vendorSha256 = null;
        doCheck = true;
        dontFixup = true;
        ldflags = [
          "-v -extldflags '-L${wasmvm}/lib'"
        ];
      };

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
