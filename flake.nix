{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    arion = {
      url = "github:hercules-ci/arion";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    foundry = {
      url = "github:shazow/foundry.nix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems =
        [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      imports = [
        ./uniond/uniond.nix
        ./uniond/proto.nix
        ./docs/docs.nix
        ./genesis/genesis.nix
        ./devnet.nix
        ./evm/evm.nix
        inputs.treefmt-nix.flakeModule
        inputs.pre-commit-hooks.flakeModule
      ];
      perSystem = { config, self', inputs', pkgs, system, lib, ... }: {
        _module = {
          args = {
            devnetConfig = { validatorCount = 4; };
          };
        };

        packages = { default = self'.packages.uniond; };

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

          nil = pkgs.stdenv.mkDerivation {
            name = "nil";
            dontUnpack = true;
            src = ./.;
            buildInputs = [ pkgs.nil ];
            doCheck = true;
            checkPhase = ''
              cd $src/.
              for i in `find . -name "*.nix" -type f`; do
                  nil diagnostics "$i"
              done
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
                pass_filenames = false;
              };
              spellcheck = {
                enable = true;
                name = "spellcheck";
                entry = "nix build .#checks.${system}.spellcheck -L";
                pass_filenames = false;
              };
            };
          };
        };

        devShells =
          let
            baseShell = {
              buildInputs = with pkgs; [
                protobuf
                buf
                nixfmt
                go_1_20
                gopls
                gotools
                go-tools
                nodejs
                yarn
                nil
                marksman
                jq
                yq
                solc
              ];
              nativeBuildInputs = [
                config.treefmt.build.wrapper
              ];
              GOPRIVATE = "github.com/unionfi/*";
            };
          in
          {
            default = pkgs.mkShell baseShell;
            githook = pkgs.mkShell (baseShell // {
              inherit (self'.checks.pre-commit-check) shellHook;
            });
            # @hussein-aitlahcen: require `--option sandbox relaxed`
            evm = pkgs.mkShell (baseShell // {
              buildInputs = baseShell.buildInputs
                ++ [
                inputs.foundry.defaultPackage.${system}
                pkgs.solc
                pkgs.go-ethereum
                self'.packages.lodestar-cli
              ];
            });
          };

        treefmt = {
          projectRootFile = "flake.nix";
          programs.nixpkgs-fmt.enable = true;
          programs.gofmt.enable = true;
          settings.global.excludes = [ "uniond/vendor/**" ];
        };
      };
    };
}
