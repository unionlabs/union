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
      url = "github:ignite/cli/v0.26.1";      
    };
  };
  outputs = inputs@{ self, nixpkgs, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems =
        [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      perSystem = { config, self', inputs', pkgs, system, lib, ... }: rec {
        packages = {
          # ignite cli package for build/devshell
          ignite-cli = pkgs.buildGoModule rec {
            allowGoReference = true;
            patches = [
              ./patches/protoc.patch
            ];
            nativeBuildInputs = [pkgs.protobuf];
            buildInputs = [pkgs.protobuf];
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

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            self'.packages.ignite-cli
            protobuf
            nixfmt
            go
            gopls
            nodejs
          ];
          PROTOC="${pkgs.protobuf}/bin/protoc";
        };
        
        apps = {
          ignite-cli.program = "${config.packages.ignite-cli}/bin/ignite";
        };
      };
    };
}
