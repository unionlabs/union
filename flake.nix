{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    ignite-cli-src = {
      flake = false;
      url = "github:ignite/cli/4098ae9a5941fd1875c8eb62540482076bd6f6d6";
    };

    swagger-combine-src = {
      flake = false;
      url = "github:maxdome/swagger-combine";
    };

    treefmt-nix.url = "github:numtide/treefmt-nix";
  };
  outputs = inputs@{ self, nixpkgs, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems =
        [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      imports = [ inputs.treefmt-nix.flakeModule ];
      perSystem = { config, self', inputs', pkgs, system, lib, ... }: rec {
        packages = rec {
          # ignite cli package for build/devshell
          swagger-combine = pkgs.buildNpmPackage {
            pname = "swagger-combine";
            version = "10.0.9";
            src = inputs.swagger-combine-src;
            dontNpmBuild = true;
            npmDepsHash = "sha256-FZR8hefkqTwSZJMX4lzS4zk7iGXi0+zi0ol1ia3iLYs=";
          };

          uniond = pkgs.buildGoModule rec {
            name = "uniond";
            src = ./uniond;
            vendorSha256 = null;
            doCheck = true;
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
            vendorSha256 = "sha256-TWOxdq2LTnxd718Ra0viD1z2tBnNmcN92A1wpX97xtc=";
            doCheck = false;
            ldflags = ''
              -X github.com/ignite/cli/ignite/version.Head=${src.rev}
              -X github.com/ignite/cli/ignite/version.Version=v0.26.1
              -X github.com/ignite/cli/ignite/version.Date=${builtins.toString (src.lastModified)}
            '';
          };
          
          docs-package = pkgs.buildNpmPackage {
            name = "docs-package";
            src = ./docs/.;
            buildPhase = ''
              npm run build
            '';
            npmDepsHash = "sha256-j/i0MM+kzvcsZs8aWab6xdHJ+QSW0S1MQcS+A2RiTY0=";
            installPhase = ''
              mkdir -p $out
              cp -dR ./build $out
            '';
          };
          
          default = uniond;
        };


        checks = {
          go-test = pkgs.go.stdenv.mkDerivation {
            name = "go-test";
            buildInputs = [ pkgs.go ];
            src = ./uniond;
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
            src = ./uniond;
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
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            self'.packages.ignite-cli
            protobuf
            nixfmt
            go
            gopls
            gotools
            go-tools
            nodejs
          ];
          nativeBuildInputs = [
            config.treefmt.build.wrapper
          ];
          PROTOC = "${pkgs.protobuf}/bin/protoc";
          SWAGGER_BIN = "${self'.packages.swagger-combine}/bin/swagger-combine";
        };

        treefmt = {
          projectRootFile = "flake.nix";
          programs.nixpkgs-fmt.enable = true;
          programs.gofmt.enable = true;
          settings.global.excludes = [
            "uniond/vendor/**"
          ];
        };

        apps = {
          ignite-cli.program = "${config.packages.ignite-cli}/bin/ignite";
          
          docs = {
            type = "app";
            program = pkgs.writeShellApplication {
              name = "docs";
              runtimeInputs = [ pkgs.nodejs ];
              text = ''
                cd docs
                npm install
                npm run start
              '';
            };
          };
        };
      };
    };
}
