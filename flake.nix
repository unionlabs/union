{
  description =
    "Union is a trust-minimized, zero-knowledge bridging protocol, designed for censorship resistance, extremely high security and usage in decentralized finance.";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?rev=75a5ebf473cd60148ba9aec0d219f72e5cf52519";
    # Track a separate nixpkgs for latest solc
    nixpkgs-solc.url = "github:NixOS/nixpkgs/nixos-unstable";
    # We need the latest nixpkgs for buildGo121Module, remove this once we upgrade nixpkgs
    nixpkgs-go.url = "github:NixOS/nixpkgs/nixos-unstable";
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
      url =
        "github:cosmos/ibc-go?rev=c98311964dc550b9fe9a5bff8b6dd8e35bf13642";
      flake = false;
    };
    ics23 = {
      url = "github:cosmos/ics23?rev=bf89d957b019bb9a2f381edb1f24d06957807690";
      flake = false;
    };
    cosmosproto = {
      url =
        "github:cosmos/cosmos-proto?rev=78e33f25b874e7639f540037599d8ea1d161a62c";
      flake = false;
    };
    gogoproto = {
      url =
        "github:cosmos/gogoproto?rev=b12c8cae0624d2518ab995c775410694dfa5d50e";
      flake = false;
    };
    googleapis = {
      url =
        "github:googleapis/googleapis?rev=6774ccbbc3f182f6ae3a32dca29e1da489ad8a8f";
      flake = false;
    };
    wasmd = {
      url = "github:CosmWasm/wasmd?rev=03f3c72a6ce447fafc2da023a1322899327433f8";
      flake = false;
    };
    nix-filter.url = "github:numtide/nix-filter";
    get-flake.url = "github:ursi/get-flake";
    # uniond versions
    v0_14_0 = {
      # NOTE: This *must* be after this commit
      url = "github:unionlabs/union/14007cbae6d464e70ca68220a36b30cb445f82fd";
      flake = false;
    };
    v0_15_0 = {
      url = "github:unionlabs/union/release-v0.15.0";
      flake = false;
    };
    v0_16_0 = {
      url = "github:unionlabs/union/release-v0.16.0";
      flake = false;
    };
    v0_17_0 = {
      url = "github:unionlabs/union/release-v0.17.0";
      flake = false;
    };
    v0_18_0 = {
      url = "github:unionlabs/union/release-v0.18.0";
      flake = false;
    };
  };
  outputs =
    inputs@{ self
    , nixpkgs
    , nixpkgs-solc
    , nixpkgs-go
    , flake-parts
    , nix-filter
    , crane
    , foundry
    , treefmt-nix
    , iohk-nix
    , ibc-go
    , ics23
    , cosmosproto
    , gogoproto
    , googleapis
    , get-flake
    , wasmd
    , ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems =
        [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      imports = [
        ./uniond/uniond.nix
        ./galoisd/galoisd.nix
        ./unionvisor/unionvisor.nix
        ./voyager/voyager.nix
        ./lib/ics23/ics23.nix
        ./hubble/hubble.nix
        ./lib/ethereum-verifier/ethereum-verifier.nix
        ./uniond/proto.nix
        ./typescript-sdk/typescript-sdk.nix
        ./site/site.nix
        ./light-clients/ethereum-light-client/ethereum-light-client.nix
        ./light-clients/cometbls-light-client/cometbls-light-client.nix
        ./lib/cometbls-groth16-verifier/default.nix
        ./cosmwasm/cosmwasm.nix
        ./evm/evm.nix
        ./tools/rust-proto.nix
        ./tools/wasm-light-client.nix
        ./tools/vendor.nix
        ./tools/generate-rust-sol-bindings/generate-rust-sol-bindings.nix
        ./tools/libwasmvm/libwasmvm.nix
        ./tools/rust/rust.nix
        ./tools/rust/crane.nix
        ./tools/tera/tera.nix
        ./tools/oxlint/oxlint.nix
        ./tools/docgen/docgen.nix
        ./tools/hasura-cli/hasura-cli.nix
        ./tools/todo-comment.nix
        ./tools/iaviewer/iaviewer.nix
        ./networks/e2e-setup.nix
        ./networks/devnet.nix
        ./networks/genesis/devnet-minimal.nix
        ./networks/genesis/devnet.nix
        ./networks/simulation/simd.nix
        ./networks/simulation/genesis.nix
        ./testnet-validator.nix
        ./e2e/all-tests.nix
        ./e2e/e2e.nix
        ./lib/unionlabs/fuzz/default.nix
        ./faucet/faucet.nix
        ./ucli/ucli.nix
        ./zerg/zerg.nix
        treefmt-nix.flakeModule
      ];

      perSystem =
        { config
        , self'
        , inputs'
        , pkgs
        , treefmt
        , rust
        , crane
        , system
        , lib
        , oxlint
        , ...
        }:
        let
          mkUnpack = import ./tools/mkUnpack.nix { inherit pkgs; };
          dbg = value:
            builtins.trace (pkgs.lib.generators.toPretty { } value) value;

          versions = builtins.fromJSON (builtins.readFile ./versions.json);

          uniondBundleVersions = rec {
            complete = versions.union-testnet-5;
            first = pkgs.lib.lists.head complete;
            last = pkgs.lib.lists.last complete;
          };

          goPkgs = import inputs.nixpkgs-go { inherit system; };
        in
        {
          _module = {
            args = {
              inherit nixpkgs dbg get-flake uniondBundleVersions goPkgs;

              gitRev =
                if (builtins.hasAttr "rev" self)
                then self.rev
                else "dirty";

              writeShellApplicationWithArgs = import ./tools/writeShellApplicationWithArgs.nix { inherit pkgs; };

              pkgs = nixpkgs.legacyPackages.${system}.appendOverlays
                (with inputs; [
                  rust-overlay.overlays.default
                  iohk-nix.overlays.crypto
                  foundry.overlay
                  (_: _: {
                    solc =
                      let
                        jsoncppVersion = "1.9.3";
                        jsoncppUrl = "https://github.com/open-source-parsers/jsoncpp/archive/${jsoncppVersion}.tar.gz";
                        jsoncpp = pkgs.fetchzip {
                          url = jsoncppUrl;
                          sha256 = "1vbhi503rgwarf275ajfdb8vpdcbn1f7917wjkf8jghqwb1c24lq";
                        };
                        range3Version = "0.12.0";
                        range3Url = "https://github.com/ericniebler/range-v3/archive/${range3Version}.tar.gz";
                        range3 = pkgs.fetchzip {
                          url = range3Url;
                          sha256 = "sha256-bRSX91+ROqG1C3nB9HSQaKgLzOHEFy9mrD2WW3PRBWU=";
                        };
                        fmtlibVersion = "9.1.0";
                        fmtlibUrl = "https://github.com/fmtlib/fmt/archive/${fmtlibVersion}.tar.gz";
                        fmtlib = pkgs.fetchzip {
                          url = fmtlibUrl;
                          sha256 = "1mnvxqsan034d2jiqnw2yvkljl7lwvhakmj5bscwp1fpkn655bbw";
                        };
                      in
                      nixpkgs-solc.legacyPackages.${system}.solc.overrideAttrs (old: old // rec {
                        version = "0.8.23";
                        src = pkgs.fetchzip {
                          url = "https://github.com/ethereum/solidity/releases/download/v${version}/solidity_${version}.tar.gz";
                          sha256 = "sha256-9GIDfjkjDFrZQ0uqopDycMWYUN+M9yLF9NpOgSksXqI=";
                        };
                        postPatch = ''
                          substituteInPlace cmake/jsoncpp.cmake \
                            --replace "${jsoncppUrl}" ${jsoncpp}
                          substituteInPlace cmake/range-v3.cmake \
                            --replace "${range3Url}" ${range3}
                          substituteInPlace cmake/fmtlib.cmake \
                            --replace "${fmtlibUrl}" ${fmtlib}
                        '';
                      });
                  })
                ]);

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
                      jailed_validator_threshold = "10";
                    };
                    slashing.params = { signed_blocks_window = "10"; };
                  };
                };
                validatorCount = 4;
                ethereum = { beacon = { validatorCount = 128; }; };
              };

              nix-filter = nix-filter.lib;

              proto = {
                wasmd = mkUnpack {
                  name = "wasmd-proto";
                  package = wasmd;
                };
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
                  rev = "0005bda13742d508487f066ba6fa3cb91495fc99";
                  allRefs = true;
                };
                cosmossdk = builtins.fetchGit {
                  name = "cosmos-sdk";
                  url = "git@github.com:unionlabs/cosmos-sdk";
                  rev = "f24b2ba59f6ec8fb4624e1b2774ad7f2d90936a2";
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
          };

          devShells.default = pkgs.mkShell {
            name = "union-devShell";
            buildInputs = [ rust.toolchains.dev oxlint ] ++ (with pkgs; [
              cargo-llvm-cov
              bacon
              cargo-nextest
              jq
              go-ethereum
              marksman
              nil
              nixfmt
              nix-tree
              nodejs_20
              openssl
              pkg-config
              protobuf
              self'.packages.tdc
              yq
              nodePackages.graphqurl
              nodePackages.svelte-language-server
              nodePackages.typescript-language-server
              nodePackages.vscode-css-languageserver-bin
            ] ++ (with goPkgs; [
              go
              gopls
              go-tools
              gotools
            ]) ++ (if pkgs.stdenv.isLinux then [
              pkgs.solc
              pkgs.foundry-bin
              pkgs.sqlx-cli
              self'.packages.hasura-cli
            ] else [ ]));
            nativeBuildInputs = [ config.treefmt.build.wrapper ]
              ++ lib.attrsets.attrValues config.treefmt.build.programs;
            GOPRIVATE = "github.com/unionlabs/*";

            shellHook = ''
              alias voy-send-msg='curl localhost:65534/msg -H "content-type: application/json" -d'
            '';
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
                npmDepsHash =
                  "sha256-Hzc4j9icNxTJNNaZ3PrmLKcUVR26nu4KqLireP4WmZM=";
              };
            in
            {
              projectRootFile = "flake.nix";
              programs = {
                nixpkgs-fmt.enable = true;
                gofmt = {
                  enable = true;
                  package = goPkgs.go;
                };
                rustfmt = {
                  enable = true;
                  package = rust.toolchains.dev;
                };
                sort = {
                  enable = true;
                  file = "dictionary.txt";
                };
                prettier.enable = true;
                taplo = {
                  enable = true;
                };
              };
              settings = {
                global.excludes = [ "**/vendor/**" ];
                formatter.prettier = {
                  # TODO: Use settings.pluginSearchDirs
                  options = [ "--write" ] ++ (if pkgs.stdenv.isLinux then [ "--plugin-search-dir=${prettier-solidity}/lib" ] else [ ]);
                  includes = [
                    "*.css"
                    "*.html"
                    "*.js"
                    "*.cjs"
                    "*.mjs"
                    "*.json"
                    "*.jsx"
                    "*.md"
                    "*.mdx"
                    "*.scss"
                    "*.ts"
                    "*.tsx"
                    "*.d.ts"
                    "*.yaml"
                    "*.yml"
                    "*.sol"
                  ];
                };
              };
            };
        };
    };

  nixConfig = {
    extra-substituters = [ "https://union.cachix.org/" ];
    extra-trusted-public-keys =
      [ "union.cachix.org-1:TV9o8jexzNVbM1VNBOq9fu8NK+hL6ZhOyOh0quATy+M=" ];
  };
}
