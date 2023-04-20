{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    ignite-cli-src.flake = false;
    ignite-cli-src.url = "github:ignite/cli/v0.24.0";
  };
  outputs = inputs@{ self, nixpkgs, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems =
        [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      perSystem = { config, self', inputs', pkgs, system, lib, ... }: {
        packages = rec {
          # ignite cli package for build/devshell
          ignite-cli = pkgs.buildGoModule rec {
            name = "ignite-cli";
            src = inputs.ignite-cli-src;
            vendorSha256 = "sha256-P1NYgvdobi6qy1sSKFwkBwPRpLuvCJE5rCD2s/vvm14=";
            doCheck = false;
            ldflags = ''
              -X github.com/ignite/cli/ignite/version.Head=${src.rev}
              -X github.com/ignite/cli/ignite/version.Version=v0.24.0
              -X github.com/ignite/cli/ignite/version.Date=${builtins.toString (src.lastModified)}
            '';
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            nixfmt
            go
            gopls
            nodejs
          ];
        };
        
        apps = {
          ignite-cli.program = "${config.packages.ignite-cli}/bin/ignite";
        };
      };
    };
}
