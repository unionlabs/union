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
  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems =
        [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      imports = [
        inputs.treefmt-nix.flakeModule
        ./tools/ignite-cli/ignite-cli.nix
        ./tools/swagger-combine/swagger-combine.nix

        ./uniond/uniond.nix

        ./docs/docs.nix
      ];
      perSystem = { config, self', inputs', pkgs, system, lib, ... }: {
        packages = {
          default = self'.packages.uniond;
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
            nil
            marksman
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
        };
      };
    };
}
