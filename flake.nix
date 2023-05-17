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
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # uniond versions
    "v0.0.2".url = "git+https://github.com/unionfi/union?ref=v0.0.2";
    "v0.3.0".url = "git+https://github.com/unionfi/union?ref=v0.3.0";
    "v0.4.2".url = "git+https://github.com/unionfi/union?ref=release-v0.4.2";
  };
  outputs = inputs@{ self, nixpkgs, flake-parts, crane, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems =
        [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      imports = [
        ./uniond/uniond.nix
        ./unionpd/unionpd.nix
        ./unionvisor/unionvisor.nix
        ./uniond/proto.nix
        ./docs/docs.nix
        ./rust/rust.nix
        ./evm/evm.nix
        ./tools/prysmctl.nix
        ./tools/prysm-beacon-chain.nix
        ./tools/prysm-validator.nix
        ./rust/proto.nix
        ./tools/libwasmvm/libwasmvm.nix
        ./networks/devnet.nix
        ./networks/genesis/devnet.nix
        inputs.treefmt-nix.flakeModule
        inputs.pre-commit-hooks.flakeModule
      ];
      perSystem = { config, self', inputs', pkgs, system, lib, ... }:
        let
          crane = rec {
            lib = self.inputs.crane.lib.${system};
            nightly = lib.overrideToolchain self'.packages.rust-nightly;
          };
        in
        {
          _module = {
            args = {
              inherit crane;
              pkgs = import nixpkgs {
                inherit system;
                overlays = with inputs; [
                  rust-overlay.overlays.default
                ];
              };
              devnetConfig = {
                validatorCount = 4;
                ethereum = {
                  beacon = {
                    validatorCount = 64;
                  };
                };
              };
              proto = {
                uniond = builtins.path {
                  name = "uniond-proto";
                  path = ./uniond/proto;
                };
                unionpd = builtins.path {
                  name = "unionpd-proto";
                  path = ./unionpd/proto;
                };
                cometbls = builtins.fetchGit {
                  name = "cometbls";
                  url = "github:UnionFi/cometbls";
                  rev = "f19ae296cf176b343ea214967810ba735813e73f";
                };
                cosmossdk = builtins.fetchGit {
                  name = "cosmos-sdk";
                  url = "github:UnionFi/cosmos-sdk";
                  rev = "021566a5aba49e79356e2e6e246494e118f12605";
                };
                ibcgo = pkgs.fetchFromGitHub {
                  name = "ibc-go";
                  owner = "strangelove-ventures";
                  repo = "ibc-go";
                  rev = "f8081a1828e47e11791b036659dd6d0e7be5473b";
                  sha256 = "sha256-e9z9+VxoQkrvWeYzdxHax6L10eQebRjW7GrD5wnaLv8=";
                };
                ics23 = pkgs.fetchFromGitHub {
                  name = "ics23";
                  owner = "cosmos";
                  repo = "ics23";
                  rev = "b1abd8678aab07165efd453c96796a179eb3131f";
                  sha256 = "sha256-O7oZI+29xKAbMHssg5HhxlssedSfejCuzHNHYX7WwBc=";
                };
                cosmosproto = pkgs.fetchFromGitHub {
                  name = "cosmosproto";
                  owner = "cosmos";
                  repo = "cosmos-proto";
                  rev = "v1.0.0-beta.3";
                  sha256 = "sha256-kFm1ChSmm5pU9oJqKmWq4KfO/hxgxzvcSzr66oTulos=";
                };
                gogoproto = pkgs.fetchFromGitHub {
                  name = "gogoproto";
                  owner = "cosmos";
                  repo = "gogoproto";
                  rev = "v1.4.7";
                  sha256 = "sha256-oaGwDFbz/xgL7hDtvdh/mIcRIGBdp+/xuKeuBE2ZpqY=";
                };
                googleapis = pkgs.fetchFromGitHub {
                  name = "googleapis";
                  owner = "googleapis";
                  repo = "googleapis";
                  rev = "6774ccbbc3f182f6ae3a32dca29e1da489ad8a8f";
                  sha256 = "sha256-TME4wkdmqrb0Shuc5uFqSGSoDaMhM9YJv9kvTam7c9I=";
                };
              };
            };
          };

          packages = {
            default = self'.packages.uniond;
            rust-nightly = pkgs.rust-bin.fromRustupToolchainFile ./rust/rust-toolchain.toml;
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
                buildInputs = [ self'.packages.rust-nightly ] ++
                  (with pkgs; [
                    protobuf
                    pkg-config
                    openssl
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
                    self'.packages.rust-stable
                  ]);
                nativeBuildInputs = [ config.treefmt.build.wrapper ];
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
                buildInputs = baseShell.buildInputs ++ [
                  inputs.foundry.defaultPackage.${system}
                  pkgs.solc
                  pkgs.go-ethereum
                  self'.packages.prysmctl
                  self'.packages.prysm-beacon-chain
                  self'.packages.prysm-validator
                ];
              });
            };

          treefmt = {
            projectRootFile = "flake.nix";
            programs.nixpkgs-fmt.enable = true;
            programs.gofmt.enable = true;
            programs.rustfmt.enable = true;
            settings.global.excludes = [ "uniond/vendor/**" ];
          };
        };
    };
}
