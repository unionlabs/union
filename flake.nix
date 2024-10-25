{
  description = "Union is a trust-minimized, zero-knowledge bridging protocol, designed for censorship resistance, extremely high security and usage in decentralized finance.";
  inputs = {
    solc = {
      url = "github:hellwolf/solc.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs?rev=75a5ebf473cd60148ba9aec0d219f72e5cf52519";
    # Track a separate nixpkgs for latest solc
    nixpkgs-solc.url = "github:NixOS/nixpkgs/nixos-unstable";
    # We need the latest nixpkgs for buildGo121Module, remove this once we upgrade nixpkgs
    nixpkgs-go.url = "github:NixOS/nixpkgs/nixos-23.11";
    # Track a separate nixpkgs for unstable nixos
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
    # Remove when lnav is updated on upstream nixpkgs
    nixpkgs-lnav.url = "github:cor/nixpkgs/lnav-v0.12.2-beta";
    process-compose.url = "github:F1bonacc1/process-compose";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    arion = {
      url = "github:hercules-ci/arion?rev=6a1f03329c400327b3b2e0ed5e1efff11037ba67";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
    };
    foundry = {
      url = "github:shazow/foundry.nix/main";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
    };
    env-utils = {
      url = "github:oceanlewis/env-utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # Prysm bls12-381 native for eth LC aggregate/verify custom query
    blst = {
      url = "github:supranational/blst?rev=3dd0f804b1819e5d03fb22ca2e6fac105932043a";
      flake = false;
    };
    bls-eth-go = {
      url = "git+https://github.com/herumi/bls-eth-go-binary?ref=refs/tags/v1.33.0&submodules=1";
      flake = false;
    };

    ibc-go = {
      url = "github:cosmos/ibc-go?rev=c98311964dc550b9fe9a5bff8b6dd8e35bf13642";
      flake = false;
    };
    ics23 = {
      url = "github:cosmos/ics23";
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
    wasmd = {
      url = "github:CosmWasm/wasmd?rev=7b418de3f6cf8fbac1e9cb11c57983fcc17264d0";
      flake = false;
    };
    nix-filter.url = "github:numtide/nix-filter?rev=3449dc925982ad46246cfc36469baf66e1b64f17";
    get-flake.url = "github:ursi/get-flake";
    wasmvm = {
      url = "github:CosmWasm/wasmvm/v1.5.2";
      flake = false;
    };
    wasmvm-1_5_0 = {
      url = "github:CosmWasm/wasmvm/v1.5.0";
      flake = false;
    };
    wasmvm-2_0_1 = {
      url = "github:CosmWasm/wasmvm/v2.0.1";
      flake = false;
    };
    biome = {
      url = "github:biomejs/biome/cli/v1.9.3";
      flake = false;
    };

    stargaze = {
      url = "git+https://github.com/public-awesome/stargaze?ref=main&submodules=1";
      flake = false;
    };
    osmosis = {
      url = "github:osmosis-labs/osmosis/v24.0.0-rc0";
      flake = false;
    };

    public-awesome-launchpad = {
      type = "github";
      owner = "public-awesome";
      repo = "launchpad";
      rev = "a14454cd2ee914af5ce10cd4cc94a9b6bfe660f6";
      flake = false;
    };
    cosmwasm-nfts = {
      type = "github";
      owner = "hussein-aitlahcen";
      repo = "cw-nfts";
      rev = "f2d7a07df63504ff8cbf0aad4140e56b3f5bfc3e";
      flake = false;
    };
    cometbls = {
      type = "github";
      owner = "unionlabs";
      repo = "cometbls";
      rev = "360766577f7daa89f958a4c28eee909340eb4b02";
      flake = false;
    };
    cosmossdk = {
      type = "github";
      owner = "unionlabs";
      repo = "cosmos-sdk";
      rev = "7d067955f7028f45b3ce205b5c35aab2e1946b19";
      flake = false;
    };

    ethereum-consensus-specs = {
      url = "https://github.com/ethereum/consensus-spec-tests/releases/download/v1.4.0/general.tar.gz";
      flake = false;
    };

    # uniond versions
    v0_21_0 = {
      url = "github:unionlabs/union/release-v0.21.0";
      flake = false;
    };
    v0_22_0 = {
      url = "github:unionlabs/union/release-v0.22.0";
      flake = false;
    };
    v0_23_0 = {
      url = "github:unionlabs/union/release-v0.23.0";
      flake = false;
    };
    v0_24_0 = {
      url = "github:unionlabs/union/release-v0.24.0";
      flake = false;
    };
  };
  outputs =
    inputs@{
      self,
      nixpkgs,
      nixpkgs-solc,
      nixpkgs-go,
      flake-parts,
      nix-filter,
      foundry,
      treefmt-nix,
      ibc-go,
      ics23,
      cosmosproto,
      gogoproto,
      googleapis,
      get-flake,
      wasmd,
      solc,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      flake = {
        site = {
          x86_64-linux = {
            inherit (self.packages.x86_64-linux) site;
            inherit (self.packages.x86_64-linux) app;
            inherit (self.packages.x86_64-linux) ceremony;
          };
          aarch64-linux = {
            inherit (self.packages.aarch64-linux) site;
            inherit (self.packages.aarch64-linux) app;
            inherit (self.packages.aarch64-linux) ceremony;
          };
        };
      };
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
      imports = [
        ./devShell.nix
        ./uniond/uniond.nix
        ./galoisd/galoisd.nix
        ./unionvisor/unionvisor.nix
        ./voyager/voyager.nix
        ./lib/ics23/ics23.nix
        ./lib/ssz/ssz.nix
        ./lib/unionlabs/unionlabs.nix
        ./hubble/hubble.nix
        ./lib/aptos.nix
        ./lib/ethereum-verifier/ethereum-verifier.nix
        ./lib/tendermint-verifier/tendermint-verifier.nix
        ./lib/scroll-verifier/scroll-verifier.nix
        ./uniond/proto.nix
        ./app/app.nix
        ./docs/docs.nix
        ./docs/openapi.nix
        ./ceremony/ceremony.nix
        ./site/site.nix
        ./lib/near/near.nix
        ./typescript-sdk/typescript-sdk.nix
        ./light-clients/ethereum-light-client/ethereum-light-client.nix
        ./light-clients/cometbls-light-client/cometbls-light-client.nix
        ./light-clients/tendermint-light-client/tendermint-light-client.nix
        ./light-clients/scroll-light-client/scroll-light-client.nix
        ./light-clients/arbitrum-light-client/arbitrum-light-client.nix
        ./light-clients/linea-light-client/linea-light-client.nix
        ./light-clients/berachain-light-client/berachain-light-client.nix
        ./light-clients/evm-in-cosmos-light-client/evm-in-cosmos-light-client.nix
        ./light-clients/movement/ics08-movement/ics08-movement.nix
        ./lib/cometbls-groth16-verifier/default.nix
        ./lib/linea-verifier/default.nix
        ./lib/linea-zktrie/default.nix
        ./cosmwasm/cosmwasm.nix
        ./evm/evm.nix
        ./tools/rust-proto.nix
        ./tools/tools.nix
        ./tools/wasm-light-client.nix
        ./tools/generate-rust-sol-bindings/generate-rust-sol-bindings.nix
        ./tools/libwasmvm/libwasmvm.nix
        ./tools/libblst/libblst.nix
        ./tools/tidy/tidy.nix
        ./tools/rust/rust.nix
        ./tools/rust/crane.nix
        ./tools/tera/tera.nix
        ./tools/biome/biome.nix
        ./tools/docgen/docgen.nix
        ./tools/hasura-cli/hasura-cli.nix
        ./tools/todo-comment.nix
        ./tools/iaviewer/iaviewer.nix
        ./networks/e2e-setup.nix
        ./networks/devnet.nix
        ./networks/simulation/simd.nix
        ./networks/stargaze.nix
        ./networks/osmosis.nix
        ./e2e/all-tests.nix
        ./e2e/e2e.nix
        ./devnet-compose/devnet-compose.nix
        ./drip/drip.nix
        ./ucli/ucli.nix
        treefmt-nix.flakeModule
      ];

      perSystem =
        {
          config,
          self',
          pkgs,
          rust,
          system,
          lib,
          biome,
          ...
        }:
        let
          mkCi = import ./tools/mkCi.nix { inherit pkgs; };
          dbg =
            value:
            builtins.trace (
              if value ? type && value.type == "derivation" then
                "derivation: ${value}"
              else
                pkgs.lib.generators.toPretty { } value
            ) value;

          versions = builtins.fromJSON (builtins.readFile ./versions/versions.json);

          uniondBundleVersions = rec {
            complete = versions.union-testnet-8.versions;
            first = pkgs.lib.lists.head complete;
            last = pkgs.lib.lists.last complete;
          };

          goPkgs = import inputs.nixpkgs-go { inherit system; };
          unstablePkgs = import inputs.nixpkgs-unstable { inherit system; };
        in
        {
          _module = {
            args = {
              inherit
                nixpkgs
                dbg
                get-flake
                uniondBundleVersions
                goPkgs
                unstablePkgs
                mkCi
                ;

              gitRev = if (builtins.hasAttr "rev" self) then self.rev else "dirty";

              pkgs = nixpkgs.legacyPackages.${system}.appendOverlays (
                with inputs;
                [
                  solc.overlay
                  rust-overlay.overlays.default
                  foundry.overlay
                  (_: super: {
                    inherit (self'.packages) devnet-utils;

                    go-ethereum = super.go-ethereum.override {
                      buildGoModule =
                        args:
                        super.buildGoModule (
                          args
                          // rec {
                            version = "1.13.12";
                            src = pkgs.fetchFromGitHub {
                              owner = "ethereum";
                              repo = "go-ethereum";
                              rev = "v${version}";
                              sha256 = "sha256-2olJV7Z01kuXlUGyI0v4YNW07/RfYiDUhBncCIS4s0A=";
                            };
                            vendorHash = "sha256-gcLVQTBpOE0DHz7/p7PENhwghftJKUDm88/4jaQ1VYw=";
                            subPackages = [
                              "cmd/abidump"
                              "cmd/abigen"
                              "cmd/bootnode"
                              "cmd/clef"
                              "cmd/devp2p"
                              "cmd/era"
                              "cmd/ethkey"
                              "cmd/evm"
                              "cmd/geth"
                              "cmd/p2psim"
                              "cmd/rlpdump"
                              "cmd/utils"
                            ];
                          }
                        );
                    };

                    writeShellApplicationWithArgs = import ./tools/writeShellApplicationWithArgs.nix {
                      pkgs = super;
                    };

                    solc =
                      if system == "aarch64-linux" then
                        super.gccStdenv.mkDerivation rec {
                          pname = "solc";
                          version = "0.8.27";
                          src = pkgs.fetchurl {
                            url = "https://github.com/nikitastupin/solc/raw/main/linux/aarch64/solc-v${version}";
                            hash = "sha256-L5W7foccAyGJmcvINqByiDMJUYPuy0AOaVWKDvahCac=";
                          };
                          dontUnpack = true;
                          nativeBuildInputs = [
                            super.stdenv.cc.cc.lib
                            super.autoPatchelfHook
                          ];
                          installPhase = ''
                            runHook preInstall
                            mkdir -p $out/bin
                            cp ${src} $out/bin/solc
                            chmod +x $out/bin/solc
                            runHook postInstall
                          '';
                          meta = {
                            description = "Static binary of compiler for Ethereum smart contract language Solidity";
                            homepage = "https://github.com/ethereum/solidity";
                            license = super.lib.licenses.gpl3;
                          };
                        }
                      else
                        super.solc_0_8_27;
                  })
                ]
              );

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
                validatorCount = 4;
                ethereum = {
                  beacon = {
                    validatorCount = 128;
                  };
                };
              };

              nix-filter = nix-filter.lib;

              proto = {
                inherit
                  wasmd
                  ibc-go
                  ics23
                  cosmosproto
                  gogoproto
                  googleapis
                  ;
                uniond = ./uniond/proto;
                galoisd = ./galoisd/proto;
                inherit (inputs) cometbls;
                cometbls-lc = ./11-cometbls/proto;
                inherit (inputs) cosmossdk;
              };

              openapi = {
                uniondOpenApiYml = ./uniond/docs/static/openapi.yml;
                cometblsOpenApiYml = "${inputs.cometbls}/rpc/openapi/openapi.yaml";
                ibcGoOpenApiYml = "${inputs.ibc-go}/docs/client/swagger-ui/swagger.yaml";
              };

              # Used as the salt when executing `instantiate2` in CosmWasm.
              cw-instantiate2-salt = "61616161";
            };
          };

          packages = {
            default = mkCi false self'.packages.uniond;
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

            nil = mkCi (system == "x86_64") (
              pkgs.stdenv.mkDerivation {
                name = "nil";
                dontUnpack = true;
                src = builtins.filterSource (path: type: type != "directory" || baseNameOf path != "vendor") ./.;
                buildInputs = [ pkgs.nil ];
                doCheck = true;
                checkPhase = ''
                  cd $src/.
                  for i in `find . -name "*.nix" -type f`; do
                    nil diagnostics "$i"
                  done
                  touch $out
                '';
              }
            );
          };

          devShells.default = pkgs.mkShell {
            name = "union-devShell";
            buildInputs =
              [ rust.toolchains.dev ]
              ++ (with pkgs; [
                clang
                cargo-llvm-cov
                bacon
                cargo-nextest
                jq
                go-ethereum
                marksman
                nil
                nix-tree
                openssl
                pkg-config
                protobuf
                httpie
                self'.packages.tdc
                yq
              ])
              ++ (with unstablePkgs; [
                wasm-tools
                bun # for running TypeScript files on the fly
                postgresql
                emmet-language-server
                nodePackages.graphqurl
                nodePackages_latest.nodejs
                nodePackages_latest.svelte-language-server
                nodePackages_latest."@astrojs/language-server"
                nodePackages_latest."@tailwindcss/language-server"
                nodePackages_latest.typescript-language-server
                nodePackages_latest.vscode-langservers-extracted
              ])
              ++ (with goPkgs; [
                go
                gopls
                go-tools
                gotools
              ])
              ++ (
                if pkgs.stdenv.isLinux then
                  [
                    pkgs.solc
                    pkgs.foundry-bin
                    goPkgs.sqlx-cli
                    self'.packages.hasura-cli
                  ]
                else
                  [ ]
              );
            nativeBuildInputs = [
              config.treefmt.build.wrapper
            ] ++ lib.attrsets.attrValues config.treefmt.build.programs;

            GOPRIVATE = "github.com/unionlabs/*";
            PUPPETEER_SKIP_DOWNLOAD = 1; # avoid npm install downloading chromium
            NODE_OPTIONS = "--no-warnings"; # avoid useless warnings from nodejs
            ASTRO_TELEMETRY_DISABLED = 1;

            ICS23_TEST_SUITE_DATA_DIR = "${inputs.ics23}/testdata";
            ETHEREUM_CONSENSUS_SPECS_DIR = "${inputs.ethereum-consensus-specs}";

            RUST_SRC_PATH = "${rust.toolchains.dev}/lib/rustlib/src/rust/library";

            VOYAGER_CONFIG_FILE_PATH = "voyager/devnet-config.json";

            SQLX_OFFLINE = true;
            LIBCLANG_PATH = "${pkgs.llvmPackages_14.libclang.lib}/lib";
            PROTOC = "${pkgs.protobuf}/bin/protoc";
          };
          # https://flake.parts/options/treefmt-nix#opt-perSystem.treefmt
          treefmt = import ./treefmt.nix {
            inherit (self'.packages) movefmt;
            inherit
              pkgs
              unstablePkgs
              goPkgs
              rust
              biome
              ;
          };
        };
    };

  nixConfig = {
    extra-substituters = [
    ];
    extra-trusted-public-keys = [
    ];
  };
}
