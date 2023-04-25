{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems =
        [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      imports = [
        inputs.treefmt-nix.flakeModule
        inputs.pre-commit-hooks.flakeModule
        ./uniond/uniond.nix
        ./docs/docs.nix
      ];
      perSystem = { config, self', inputs', pkgs, system, lib, ... }: {
        packages = {
          default = self'.packages.uniond;
        };

        checks = {
          spellcheck = pkgs.stdenv.mkDerivation {
            name = "spellcheck";
            dontUnpack = true;
            src = ./.;
            buildInputs = [ pkgs.nodePackages.cspell ];
            doCheck = true;
            checkPhase = ''
              cd $src/.
              cspell lint --no-progress "**"
              touch $out
            '';
          };
          pre-commit-check = inputs.pre-commit-hooks.lib.${system}.run {
            src = ./.;
            hooks = {
              commitizen.enable = true;
              nil.enable = true;
              treefmt-nix = {
                enable = true;
                name = "treefmt";
                entry = "nix build .#checks.${system}.treefmt -L";
              };
              spellcheck = {
                enable = true;
                name = "spellcheck";
                entry = "nix build .#checks.${system}.spellcheck -L";
              };
            };
          };
        };

        devShells.default = pkgs.mkShell {
          inherit (self'.checks.pre-commit-check) shellHook;
          buildInputs = with pkgs; [
            protobuf
            nixfmt
            go_1_20
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
          GOPRIVATE = "github.com/unionfi/*";
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
