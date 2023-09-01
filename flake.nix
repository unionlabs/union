{
  description = "Union is a trust-minimized, zero-knowledge bridging protocol, designed for censorship resistance, extremely high security and usage in decentralized finance.";
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
      url = "github:unionlabs/treefmt-nix";
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
    iohk-nix = {
      url = "github:input-output-hk/iohk-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    ibc-go = {
      url = "github:strangelove-ventures/ibc-go?rev=f8081a1828e47e11791b036659dd6d0e7be5473b";
      flake = false;
    };
    ics23 = {
      url = "github:cosmos/ics23?rev=b1abd8678aab07165efd453c96796a179eb3131f";
      flake = false;
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
    nix-filter.url = "github:numtide/nix-filter";
    # uniond versions
    v0_8_0.url = "github:unionlabs/union/release-v0.8.0";
    v0_9_0.url = "github:unionlabs/union/release-v0.9.0";
    v0_10_0.url = "github:unionlabs/union/release-v0.10.0";
  };
  outputs = inputs@{ self, nixpkgs, flake-parts, nix-filter, crane, foundry, treefmt-nix, pre-commit-hooks, iohk-nix, ibc-go, ics23, cosmosproto, gogoproto, googleapis, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems =
        [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      imports = [
        ./uniond/uniond.nix
        ./galoisd/galoisd.nix
        ./unionvisor/unionvisor.nix
        ./voyager/voyager.nix
        ./hubble/hubble.nix
        ./lib/ethereum-verifier/ethereum-verifier.nix
        ./uniond/proto.nix
        ./docs/docs.nix
        ./light-clients/ethereum-light-client/ethereum-light-client.nix
        ./light-clients/cometbls-light-client/cometbls-light-client.nix
        ./lib/cometbls-groth16-verifier/default.nix
        ./cosmwasm/cosmwasm.nix
        ./evm/evm.nix
        ./tools/rust-proto.nix
        ./tools/generate-rust-sol-bindings/generate-rust-sol-bindings.nix
        ./tools/libwasmvm/libwasmvm.nix
        ./tools/rust/rust.nix
        ./tools/rust/crane.nix
        ./tools/tera/tera.nix
        ./tools/docgen/docgen.nix
        ./networks/e2e-setup.nix
        ./networks/devnet.nix
        ./networks/genesis/devnet.nix
        ./testnet-validator.nix
        ./e2e/all-tests.nix
        ./e2e/e2e.nix
        ./lib/unionlabs/fuzz/default.nix
        treefmt-nix.flakeModule
        pre-commit-hooks.flakeModule
      ];

      perSystem = { config, self', inputs', pkgs, treefmt, rust, crane, system, lib, ... }:
        let
          mkUnpack = import ./tools/mkUnpack.nix { inherit pkgs; };

          dbg = value: builtins.trace (pkgs.lib.generators.toPretty { } value) value;
        in
        {
          _module = {
            args = {
              inherit nixpkgs dbg;

              pkgs = import nixpkgs {
                inherit system;
                overlays = with inputs; [
                  rust-overlay.overlays.default
                  iohk-nix.overlays.crypto
                  foundry.overlay
                ];
              };

              ensureAtRepositoryRoot = ''
                # If the current directory contains flake.nix, then we are at the repository root
                if [[ -f flake.nix ]]
                then
                  echo "We are at the repository root. Running script..."
                else
                  echo "We are NOT at the repository root. Please cd to the repository root and try again."
                  exit 1
                fi
              '';

              devnetConfig = {
                genesisOverwrites = {
                  app_state = {
                    staking.params = {
                      epoch_length = "8";
                      jailed_validator_threshold = 10;
                    };
                    slashing.params = {
                      signed_blocks_window = 10;
                    };
                  };
                };
                validatorCount = 4;
                ethereum = {
                  beacon = {
                    validatorCount = 8;
                  };
                };
              };

              nix-filter = nix-filter.lib;

              proto = {
                uniond = builtins.path {
                  name = "uniond-proto";
                  path = ./uniond/proto;
                };
                galoisd = builtins.path {
                  name = "galoisd-proto";
                  path = ./galoisd/proto;
                };
                cometbls = builtins.fetchGit {
                  name = "cometbls";
                  url = "git@github.com:unionlabs/cometbls";
                  rev = "20834775a066a30a2115c914213229d3f5033e5e";
                };
                cosmossdk = builtins.fetchGit {
                  name = "cosmos-sdk";
                  url = "git@github.com:unionlabs/cosmos-sdk";
                  rev = "30e47ad4927bc83e40468ca5878965218c4e35c8";
                  allRefs = true;
                };
                ibcgo = mkUnpack {
                  name = "ibc-go";
                  package = ibc-go;
                };
                ics23 = mkUnpack {
                  name = "ics23";
                  package = ics23;
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

              # Used as the salt when executing `instantiate2` in CosmWasm.
              cw-instantiate2-salt = "61616161";
            };
          };

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

            pre-commit-check = pre-commit-hooks.lib.${system}.run {
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
                buildInputs = [ rust.toolchains.dev ] ++ (with pkgs; [
                  bacon
                  cargo-nextest
                  go_1_20
                  gopls
                  go-tools
                  gotools
                  jq
                  marksman
                  nil
                  nixfmt
                  nodejs
                  openssl
                  pkg-config
                  protobuf
                  solc
                  yarn
                  yq
                  hasura-cli
                ]);
                nativeBuildInputs = [
                  config.treefmt.build.wrapper
                ] ++ lib.attrsets.attrValues config.treefmt.build.programs;
                GOPRIVATE = "github.com/unionlabs/*";
              };
            in
            {
              default = pkgs.mkShell baseShell;
              githook = pkgs.mkShell (baseShell // {
                inherit (self'.checks.pre-commit-check) shellHook;
              });
              evm = pkgs.mkShell (baseShell // {
                buildInputs = baseShell.buildInputs ++ [
                  pkgs.solc
                  pkgs.foundry-bin
                  pkgs.go-ethereum
                ];
              });
            };

          treefmt =
            let
              prettier-solidity = pkgs.buildNpmPackage {
                name = "prettier-plugin-solidity";
                version = "1.1.3";
                nativeBuildInputs = [ pkgs.pkg-config pkgs.python3 ];
                src = pkgs.fetchFromGitHub {
                  owner = "prettier-solidity";
                  repo = "prettier-plugin-solidity";
                  rev = "0f0b31bd1d76626cad4ce576d89088ef23ad87f3";
                  hash = "sha256-zodOB5hARb7Jrb6d4gqmBKEFKUg0ZNZKbTN7H4vJk2w=";
                };
                npmInstallFlags = "--include=dev";
                npmDepsHash = "sha256-Hzc4j9icNxTJNNaZ3PrmLKcUVR26nu4KqLireP4WmZM=";
              };
            in
            {
              projectRootFile = "flake.nix";
              programs.nixpkgs-fmt.enable = true;
              programs.gofmt.enable = true;
              programs.rustfmt = {
                enable = true;
                package = rust.toolchains.dev;
              };
              programs.sort = {
                enable = true;
                file = "dictionary.txt";
              };
              settings.global.excludes = [ "**/vendor/**" ];
              programs.prettier.enable = true;
              settings.formatter.prettier = {
                options = [ "--write" "--plugin-search-dir=${prettier-solidity}/lib" ];
                includes = [
                  "*.css"
                  "*.html"
                  "*.js"
                  "*.json"
                  "*.jsx"
                  "*.md"
                  "*.mdx"
                  "*.scss"
                  "*.ts"
                  "*.tsx"
                  "*.yaml"
                  "*.yml"
                  "*.sol"
                ];
              };
            };
        };
    };

  nixConfig = {
    extra-substituters = [ "https://union.cachix.org/" ];
    extra-trusted-public-keys = [ "union.cachix.org-1:TV9o8jexzNVbM1VNBOq9fu8NK+hL6ZhOyOh0quATy+M=" ];
  };
}
