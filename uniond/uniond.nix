{ ... }: {
  perSystem = { pkgs, self', crane, system, ensureAtRepositoryRoot, nix-filter, ... }:
    let
      CGO_CFLAGS = "-I${pkgs.libblst}/include -I${pkgs.libblst.src}/src -I${pkgs.libblst.src}/build -I${self'.packages.bls-eth.src}/bls/include -O -D__BLST_PORTABLE__";
      CGO_LDFLAGS = "-z noexecstack -static -L${pkgs.musl}/lib -L${self'.packages.libwasmvm}/lib -L${self'.packages.bls-eth}/lib -s -w";
    in
    {
      packages = {
        bls-eth =
          let
            isAarch64 = ((builtins.head (pkgs.lib.splitString "-" system)) == "aarch64");
          in
          pkgs.pkgsStatic.stdenv.mkDerivation {
            pname = "bls-eth";
            version = "1.32.0";
            src = pkgs.fetchFromGitHub {
              owner = "herumi";
              repo = "bls-eth-go-binary";
              rev = "321cb9bb9abb3359217a69e21cf579c4419aef50";
              hash = "sha256-jYOvq3hStZjrEj8xGDirZX4ham6WfY2IPi0k+d9KzaQ=";
              fetchSubmodules = true;
            };
            nativeBuildInputs = [ pkgs.pkgsStatic.nasm ] ++ (pkgs.lib.optionals isAarch64 [ pkgs.llvmPackages_9.libcxxClang ]);
            installPhase = ''
              mkdir -p $out/lib
              ls -al bls/lib/linux/
              mv bls/lib/linux/${if isAarch64 then "arm64" else "amd64"}/*.a $out/lib
            '';
            enableParallelBuilding = true;
            doCheck = true;
          };

        # Statically link on Linux using `pkgsStatic`, dynamically link on Darwin using normal `pkgs`.
        uniond = (if pkgs.stdenv.isLinux then
          pkgs.pkgsStatic.buildGoModule
        else
          pkgs.buildGoModule) ({
          name = "uniond";
          src = nix-filter
            {
              name = "uniond-source";
              root = ./.;
              exclude = [
                (nix-filter.matchExt "nix")
                (nix-filter.matchExt "md")
              ];
            };
          vendorSha256 = null;
          doCheck = true;
          meta.mainProgram = "uniond";
        } // (
          if pkgs.stdenv.isLinux then {
            inherit CGO_CFLAGS;
            inherit CGO_LDFLAGS;
            # Statically link if we're on linux
            nativeBuildInputs = [ pkgs.musl ];
            ldflags = [
              "-linkmode external"
            ];
          } else if pkgs.stdenv.isDarwin then {
            # Dynamically link if we're on darwin by wrapping the program
            # such that the DYLD_LIBRARY_PATH includes libwasmvm
            buildInputs = [ pkgs.makeWrapper ];
            postFixup = ''
              wrapProgram $out/bin/uniond \
              --set DYLD_LIBRARY_PATH ${(pkgs.lib.makeLibraryPath [ self'.packages.libwasmvm ])};
            '';
          } else
            { }
        ));

        uniond-image = pkgs.dockerTools.buildImage {
          name = "uniond";

          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            paths = [ pkgs.coreutils pkgs.cacert ];
            pathsToLink = [ "/bin" ];
          };
          config = {
            Entrypoint = [ (pkgs.lib.getExe self'.packages.uniond) ];
            Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
          };
        };

        go-vendor =
          let
            vend = pkgs.buildGoModule {
              pname = "vend";
              version = "0.0.0";
              src = pkgs.fetchFromGitHub {
                owner = "nomad-software";
                repo = "vend";
                rev = "a1ea6c775ac230bb1a1428bb96e4306044aa944b";
                sha256 = "sha256-7AdE5qps4OMjaubt9Af6ATaqrV3n73ZuI7zTz7Kgm6w=";
              };
              vendorSha256 = null;
            };

            # must be run from a directory with vendor/
            doVendor = repos:
              if repos == [ ]
              then ''
                echo "no repositories were requested to be fully vendored, only running 'go mod vendor'"
                go mod vendor
                go mod tidy
              ''
              else ''
                TMP=$(mktemp -d)

                # vendor to a tmp dir, since vend doesn't have an output option
                go mod vendor -o "$TMP"

                # outputs to ./vendor
                vend

                # overwrite the chosen repos with their fully vendored versions
                ${
                  pkgs.lib.concatMapStrings
                    (repo:
                    ''
                      echo "fully vendoring ${repo}"

                      # https://askubuntu.com/questions/269775/mv-directory-not-empty
                      rm -r "$TMP/${repo}"/*
                      mv -fv vendor/${repo}/* "$TMP/${repo}"
                    '')
                    repos
                }

                # clear vendor, to ensure that no unwanted files are kept
                rm -r vendor/*

                # move vendor back
                mv -fv "$TMP"/* vendor

                # rm -r "$TMP"

                go mod tidy
              '';
          in
          pkgs.writeShellApplication {
            name = "go-vendor";
            runtimeInputs = [ pkgs.go vend ];
            text = ''
              ${ensureAtRepositoryRoot}

              echo "vendoring uniond..."
              cd uniond
              ${doVendor [ "github.com/supranational/blst" "github.com/herumi/bls-eth-go-binary" ]}

              echo "vendoring galoisd..."
              cd ../galoisd
              ${doVendor [ ]}
            '';
          };
      };

      checks = {
        go-test = pkgs.go.stdenv.mkDerivation {
          name = "go-test";
          buildInputs = [ pkgs.go ];
          src = ./.;
          doCheck = true;
          inherit CGO_CFLAGS;
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
          inherit CGO_CFLAGS;
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
          inherit CGO_CFLAGS;
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
