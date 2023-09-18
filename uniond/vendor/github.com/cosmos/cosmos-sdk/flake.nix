{
  description = "The Cosmos SDK is a framework for building blockchain applications.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    ignite-cli-src = {
      flake = false;
      url = "github:ignite/cli/v0.26.1";
    };
    swagger-combine-src = {
      flake = false;
      url = "github:maxdome/swagger-combine";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    cosmosproto = {
      url = "github:cosmos/cosmos-proto?rev=78e33f25b874e7639f540037599d8ea1d161a62c";
      flake = false;
    };
    gogoproto = {
      url = "github:cosmos/gogoproto?rev=b12c8cae0624d2518ab995c775410694dfa5d50e";
      flake = false;
    };
    googleapis = {
      url = "github:googleapis/googleapis?rev=6774ccbbc3f182f6ae3a32dca29e1da489ad8a8f";
      flake = false;
    };
  };

  outputs = inputs@{ nixpkgs, flake-parts, treefmt-nix, cosmosproto, gogoproto, googleapis, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems =
        [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      imports = [
        ./api/api.nix
        ./client/v2/clientv2.nix
        ./core/core.nix
        ./depinject/depinject.nix
        ./errors/errors.nix
        ./math/math.nix
        ./orm/orm.nix
        ./proto/proto.nix
        ./simapp/simapp.nix
        ./tests/tests.nix
        ./tools/rosetta/rosetta.nix
        ./tx/tx.nix
        treefmt-nix.flakeModule
      ];
      perSystem = { config, self', inputs', pkgs, system, lib, ... }:
        let
          mkUnpack = import ./tools/mkUnpack.nix { inherit pkgs; };
        in
        {
          _module = {
            args = {
              inherit nixpkgs;

              pkgs = import nixpkgs {
                inherit system;
              };

              proto = {
                cosmossdk = builtins.path {
                  name = "cosmos-sdk";
                  path = ./.;
                };
                cosmosproto = mkUnpack {
                  name = "cosmos-proto";
                  package = cosmosproto;
                };
                gogoproto = mkUnpack {
                  name = "gogoproto";
                  package = gogoproto;
                };
                googleapis = mkUnpack {
                  name = "googleapis";
                  package = googleapis;
                };
              };
            };
          };
          packages = {
            go-mod-vendor = pkgs.writeShellApplication {
              name = "go-mod-vendor";
              runtimeInputs = with pkgs; [ go ];
              text = ''
                # If the current directory contains flake.nix, then we are at the repository root
                if [[ -f flake.nix ]]
                then
                  echo "We are at the repository root. Running script..."
                else
                  echo "We are NOT at the repository root. Please cd to the repository root and try again."
                  exit 1
                fi
                for project in \
                  ./. \
                  ./api/ \
                  ./client/v2/ \
                  ./core/ \
                  ./depinject/ \
                  ./errors/ \
                  ./math/ \
                  ./orm/ \
                  ./simapp/ \
                  ./tests/ \
                  ./tools/rosetta/ \
                  ./tx/
                do
                  echo "Vendoring '$project'..."
                  (
                    cd $project;
                    rm -rf vendor/*;
                    go mod tidy;
                    go mod vendor;
                  )
                done
                
                echo "Done vendoring!"
              '';
            };
            # ignite cli package for build/devshell
            swagger-combine = pkgs.buildNpmPackage {
              pname = "swagger-combine";
              version = "10.0.9";
              src = inputs.swagger-combine-src;
              dontNpmBuild = true;
              npmDepsHash = "sha256-FZR8hefkqTwSZJMX4lzS4zk7iGXi0+zi0ol1ia3iLYs=";
            };

            ignite-cli = pkgs.buildGoModule rec {
              allowGoReference = true;
              patches = [
                ./patches/protoc.patch
              ];
              nativeBuildInputs = [ pkgs.protobuf ];
              buildInputs = [ pkgs.protobuf ];
              name = "ignite-cli";
              src = inputs.ignite-cli-src;
              vendorSha256 = "sha256-4ajrp1UQ6mF75ZnZ69Y3QtRbsEsdDNm0J6pVZG+EwiY=";
              doCheck = false;
              ldflags = ''
                -X github.com/ignite/cli/ignite/version.Head=${src.rev}
                -X github.com/ignite/cli/ignite/version.Version=v0.26.1
                -X github.com/ignite/cli/ignite/version.Date=${builtins.toString (src.lastModified)}
              '';
            };
          };

          packages = {
            default = self'.packages.simapp;
          };

          devShells.default = pkgs.mkShell {
            buildInputs = with pkgs; [
              buf
              go
              gopls
              gotestsum
              go-tools
              gotools
              jq
              marksman
              nil
              nixfmt
              nodejs
              openssl
              protobuf
              self'.packages.ignite-cli
              yarn
              yq
            ];
            nativeBuildInputs = [ config.treefmt.build.wrapper ];
            PROTOC = "${pkgs.protobuf}/bin/protoc";
            SWAGGER_BIN = "${self'.packages.swagger-combine}/bin/swagger-combine";
            GOPRIVATE = "github.com/unionlabs/*";
          };

          treefmt = {
            projectRootFile = "flake.nix";
            programs.nixpkgs-fmt.enable = true;
            programs.gofumpt.enable = true;
            settings.global.excludes = [ "./vendor*" "*.git*" "./client/docs/statik/statik.go" "./test/mocks/*" "*.pb.go" "*.pb.gw.go" "*.pulsar.go" "./crypto/keys/secp256k1/*" ];
          };

          checks = {
            go-test = pkgs.go.stdenv.mkDerivation {
              name = "go-test";
              buildInputs = [ pkgs.go pkgs.gotestsum pkgs.git pkgs.which ];
              src = ./.;
              doCheck = true;
              dontInstall = true;
              dontBuild = true;
              checkPhase = ''
                # Go will try to create a .cache/ dir in $HOME.
                # We avoid this by setting $HOME to the builder directory
                export HOME=$(pwd)
                export GOFLAGS="-mod=vendor -tags=\"cgo,ledger,test_ledger_mock,norace\""
                export GOPRIVATE="github.com/unionlabs/*";
                export GOWORK="off"

                go version

                echo "Starting tests for github.com/cosmos/cosmos-sdk"

                gotestsum  ./...

                echo "Finished running sub module tests."

                touch $out
              '';
            };
          };

          apps = {
            ignite-cli.program = "${config.packages.ignite-cli}/bin/ignite";
          };
        };
    };
}
