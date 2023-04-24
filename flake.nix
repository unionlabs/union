{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems =
        [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      imports = [
        inputs.treefmt-nix.flakeModule
        ./uniond/uniond.nix
        ./docs/docs.nix
      ];
      perSystem = { config, self', inputs', pkgs, system, lib, ... }: {
        packages = {
          default = self'.packages.uniond;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            protobuf
            nixfmt
            go
            gopls
            gotools
            go-tools
            nodejs
            nil
            marksman
          ];
          nativeBuildInputs = [
            config.treefmt.build.wrapper
          ];
        };

        treefmt = {
          projectRootFile = "flake.nix";
          programs.nixpkgs-fmt.enable = true;
          programs.gofmt.enable = true;
          settings.global.excludes = [
            "uniond/vendor/**"
          ];
        };
      };
    };
}
