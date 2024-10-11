{
  description = "A Framework for Building High Value Public Blockchains";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    swagger-combine-src = {
      flake = false;
      url = "github:maxdome/swagger-combine";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    protoc-gen-go-cosmos-orm = {
      url = "github:cosmos/cosmos-sdk?rev=cd45ab2abdd9817a0ebb583f5334514e3f900cfb";
      flake = false;
    };
    cosmosproto = {
      url = "github:cosmos/cosmos-proto?rev=0748a2ad4a5c78b1db6c8090db01e255bcc91365";
      flake = false;
    };
    gogoproto = {
      url = "github:cosmos/gogoproto?rev=b12c8cae0624d2518ab995c775410694dfa5d50e";
      flake = false;
    };
    googleapis = {
      url = "github:googleapis/googleapis?rev=8984ddb508dea0e673b724c58338e810b1d8aee3";
      flake = false;
    };
    cometbft = {
      url = "github:unionlabs/cometbft?rev=8e5f9640051521f4adc92137d40568f9e7f0d3e4";
      flake = false;
    };
  };

  outputs = inputs@{ nixpkgs, flake-parts, treefmt-nix, protoc-gen-go-cosmos-orm, cosmosproto, gogoproto, googleapis, cometbft, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems =
        [ "x86_64-linux" "aarch64-linux" ];
      imports = [
        ./client/v2/clientv2.nix
        ./proto/proto.nix
        ./simapp/simapp.nix
        ./tests/tests.nix
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

              rev =
                if (builtins.hasAttr "rev" self') then self'.rev else "dev";

              proto = {
                protoc-gen-go-cosmos-orm = mkUnpack {
                  name = "protoc-gen-go-cosmos-orm";
                  package = protoc-gen-go-cosmos-orm;
                };
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
                cometbft = mkUnpack {
                  name = "cometbft";
                  package = cometbft;
                };
              };
            };
          };
          packages = {
            go-mod-vendor = pkgs.writeShellApplication {
              name = "go-mod-vendor";
              runtimeInputs = with pkgs; [ go_1_23 ];
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
                  ./client/v2/ \
                  ./simapp/ \
                  ./tests/
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
            swagger-combine = pkgs.buildNpmPackage {
              pname = "swagger-combine";
              version = "10.0.9";
              src = inputs.swagger-combine-src;
              dontNpmBuild = true;
              npmDepsHash = "sha256-FZR8hefkqTwSZJMX4lzS4zk7iGXi0+zi0ol1ia3iLYs=";
            };
          };

          packages = {
            default = self'.packages.simapp;
          };

          devShells.default = pkgs.mkShell {
            buildInputs = with pkgs; [
              buf
              go_1_23
              gopls
              gotestsum
              go-tools
              gotools
              jq
              marksman
              nil
              openssl
              protobuf
              rocksdb_8_11
            ];
            nativeBuildInputs = [ config.treefmt.build.wrapper ];
            PROTOC = "${pkgs.protobuf}/bin/protoc";
            SWAGGER_BIN = "${self'.packages.swagger-combine}/bin/swagger-combine";
            GOPRIVATE = "github.com/unionlabs/*";
          };

          treefmt = {
            projectRootFile = "flake.nix";
            programs.nixpkgs-fmt.enable = true;
            settings.global.excludes = [ "./vendor*" "*.git*" "./client/docs/statik/statik.go" "./test/mocks/*" "*.pb.go" "*.pb.gw.go" "*.pulsar.go" "./crypto/keys/secp256k1/*" ];
          };

          checks = {
            go-test = pkgs.go.stdenv.mkDerivation {
              name = "go-test";
              buildInputs = [ pkgs.go_1_23 pkgs.gotestsum pkgs.git pkgs.which ];
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
        };
    };
}

