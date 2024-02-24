{
  description =
    "Union is a trust-minimized, zero-knowledge bridging protocol, designed for censorship resistance, extremely high security and usage in decentralized finance.";
  inputs = {
    nixpkgs.url =
      "github:NixOS/nixpkgs?rev=75a5ebf473cd60148ba9aec0d219f72e5cf52519";
    # Track a separate nixpkgs for latest solc
    nixpkgs-solc.url = "github:NixOS/nixpkgs/nixos-unstable";
    # We need the latest nixpkgs for buildGo121Module, remove this once we upgrade nixpkgs
    nixpkgs-go.url = "github:NixOS/nixpkgs/nixos-23.11";
    # Track a separate nixpkgs for latest Node.js
    nixpkgs-nodejs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    arion = {
      url = "github:hercules-ci/arion?rev=6a1f03329c400327b3b2e0ed5e1efff11037ba67";
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
      url =
        "github:CosmWasm/wasmd?rev=03f3c72a6ce447fafc2da023a1322899327433f8";
      flake = false;
    };
    nix-filter.url = "github:numtide/nix-filter?rev=3449dc925982ad46246cfc36469baf66e1b64f17";
    get-flake.url = "github:ursi/get-flake?rev=e0917b6f564aa5acefb1484b5baf76da21746c3c";
    wasmvm = {
      url = "github:CosmWasm/wasmvm/v1.5.2";
      flake = false;
    };
    wasmvm-1_5_0 = {
      url = "github:CosmWasm/wasmvm/v1.5.0";
      flake = false;
    };
    oxlint = {
      url = "github:web-infra-dev/oxc/oxlint_v0.2.6";
      flake = false;
    };

    stargaze = {
      url = "git+https://github.com/public-awesome/stargaze?ref=feature/sdk-v050&submodules=1";
      flake = false;
    };
    public-awesome-launchpad = {
      type = "github";
      owner = "public-awesome";
      repo = "launchpad";
      ref = "v3.5.1";
      flake = false;
    };

    cosmossdk = {
      url = "github:unionlabs/cosmos-sdk/v0.50.2-bn254";
      flake = false;
    };
    cometbls = {
      url = "github:unionlabs/cometbls/360766577f7daa89f958a4c28eee909340eb4b02";
      flake = false;
    };

    # uniond versions
    v0_19_0 = {
      url = "github:unionlabs/union/release-v0.19.0";
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
    , blst
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
      flake =
        let
          inherit (inputs.nixpkgs.lib) filterAttrs;
          isCi = attr: v: (if v?ci then v.ci else true);
        in
        {
          build = {
            x86_64-linux = filterAttrs isCi self.packages.x86_64-linux;
            aarch64-linux = filterAttrs isCi self.packages.aarch64-linux;
          };
          test = {
            x86_64-linux = filterAttrs isCi self.checks.x86_64-linux;
            aarch64-linux = filterAttrs isCi self.checks.aarch64-linux;
          };
          dev = {
            x86_64-linux = filterAttrs isCi self.devShells.x86_64-linux;
            aarch64-linux = filterAttrs isCi self.devShells.aarch64-linux;
          };
        };
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
        ./lib/tendermint-verifier/tendermint-verifier.nix
        ./uniond/proto.nix
        ./typescript-sdk/typescript-sdk.nix
        ./app/app.nix
        ./site/site.nix
        ./light-clients/ethereum-light-client/ethereum-light-client.nix
        ./light-clients/cometbls-light-client/cometbls-light-client.nix
        ./light-clients/scroll-light-client/scroll-light-client.nix
        ./lib/cometbls-groth16-verifier/default.nix
        ./cosmwasm/cosmwasm.nix
        ./evm/evm.nix
        ./tools/rust-proto.nix
        ./tools/tools.nix
        ./tools/wasm-light-client.nix
        ./tools/generate-rust-sol-bindings/generate-rust-sol-bindings.nix
        ./tools/libwasmvm/libwasmvm.nix
        ./tools/libblst/libblst.nix
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
        ./networks/simulation/simd.nix
        ./networks/stargaze.nix
        ./testnet-validator.nix
        ./e2e/all-tests.nix
        ./e2e/e2e.nix
        ./fuzz
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
        , cargo-fuzz
        , ...
        }:
        let
          mkCi = import ./tools/mkCi.nix { inherit pkgs; };
          mkUnpack = import ./tools/mkUnpack.nix { inherit pkgs; };
          dbg = value:
            builtins.trace
              (
                if value ? type && value.type == "derivation"
                then "derivation: ${value}"
                else pkgs.lib.generators.toPretty { } value
              )
              value;

          versions = builtins.fromJSON (builtins.readFile ./versions/versions.json);

          uniondBundleVersions = rec {
            complete = versions.union-testnet-6;
            first = pkgs.lib.lists.head complete;
            last = pkgs.lib.lists.last complete;
          };

          goPkgs = import inputs.nixpkgs-go { inherit system; };
          nodePkgs = import inputs.nixpkgs-nodejs { inherit system; };
        in
        {
          _module = {
            args = {
              inherit nixpkgs dbg get-flake uniondBundleVersions goPkgs nodePkgs mkCi;

              gitRev =
                if (builtins.hasAttr "rev" self) then self.rev else "dirty";

              writeShellApplicationWithArgs =
                import ./tools/writeShellApplicationWithArgs.nix {
                  inherit pkgs;
                };

              pkgs = nixpkgs.legacyPackages.${system}.appendOverlays
                (with inputs; [
                  rust-overlay.overlays.default
                  foundry.overlay
                  (_: super: {
                    go-ethereum = super.go-ethereum.override {
                      buildGoModule = args: super.buildGoModule (args // rec {
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
                      });
                    };
                    keygen = self'.packages.keygen;
                    solc =
                      let
                        jsoncppVersion = "1.9.3";
                        jsoncppUrl =
                          "https://github.com/open-source-parsers/jsoncpp/archive/${jsoncppVersion}.tar.gz";
                        jsoncpp = pkgs.fetchzip {
                          url = jsoncppUrl;
                          sha256 =
                            "1vbhi503rgwarf275ajfdb8vpdcbn1f7917wjkf8jghqwb1c24lq";
                        };
                        range3Version = "0.12.0";
                        range3Url =
                          "https://github.com/ericniebler/range-v3/archive/${range3Version}.tar.gz";
                        range3 = pkgs.fetchzip {
                          url = range3Url;
                          sha256 =
                            "sha256-bRSX91+ROqG1C3nB9HSQaKgLzOHEFy9mrD2WW3PRBWU=";
                        };
                        fmtlibVersion = "9.1.0";
                        fmtlibUrl =
                          "https://github.com/fmtlib/fmt/archive/${fmtlibVersion}.tar.gz";
                        fmtlib = pkgs.fetchzip {
                          url = fmtlibUrl;
                          sha256 =
                            "1mnvxqsan034d2jiqnw2yvkljl7lwvhakmj5bscwp1fpkn655bbw";
                        };
                      in
                      nixpkgs-solc.legacyPackages.${system}.solc.overrideAttrs
                        (old:
                          old // rec {
                            version = "0.8.23";
                            src = pkgs.fetchzip {
                              url =
                                "https://github.com/ethereum/solidity/releases/download/v${version}/solidity_${version}.tar.gz";
                              sha256 =
                                "sha256-9GIDfjkjDFrZQ0uqopDycMWYUN+M9yLF9NpOgSksXqI=";
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
                validatorCount = 4;
                ethereum = { beacon = { validatorCount = 128; }; };
              };

              nix-filter = nix-filter.lib;

              proto = {
                inherit wasmd
                  ibc-go
                  ics23
                  cosmosproto
                  gogoproto
                  googleapis;
                uniond = ./uniond/proto;
                galoisd = ./galoisd/proto;
                cometbls = inputs.cometbls;
                cosmossdk = inputs.cosmossdk;
              };

              # Used as the salt when executing `instantiate2` in CosmWasm.
              cw-instantiate2-salt = "61616161";
            };
          };

          packages = { default = mkCi false (self'.packages.uniond); };

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

            nil = mkCi (system == "x86_64") (pkgs.stdenv.mkDerivation {
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
            });
          };

          devShells.default = pkgs.mkShell {
            name = "union-devShell";
            buildInputs = [ rust.toolchains.dev ] ++ (with pkgs; [
              cargo-fuzz
              cargo-llvm-cov
              bacon
              cargo-nextest
              jq
              go-ethereum
              marksman
              nil
              nixfmt
              nix-tree
              openssl
              pkg-config
              protobuf
              self'.packages.tdc
              yq
            ] ++ (with nodePkgs; [
              bun # for running TypeScript files on the fly
              nodejs_21
              nodePackages.graphqurl
              nodePackages.svelte-language-server
              nodePackages.typescript-language-server
              nodePackages.vscode-css-languageserver-bin
            ])
            ++ (with goPkgs; [
              go
              gopls
              go-tools
              gotools
            ]) ++ (if pkgs.stdenv.isLinux then [
              pkgs.solc
              pkgs.foundry-bin
              goPkgs.sqlx-cli
              self'.packages.hasura-cli
            ] else [ ]));
            nativeBuildInputs = [ config.treefmt.build.wrapper ]
              ++ lib.attrsets.attrValues config.treefmt.build.programs;

            GOPRIVATE = "github.com/unionlabs/*";
            PUPPETEER_SKIP_DOWNLOAD = 1; # avoid npm install downloading chromium
            NODE_OPTIONS = "--no-warnings"; # avoid useless warnings from nodejs

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
                prettier = {
                  enable = true;
                  excludes = [ "./app/**/*" ];
                };
                taplo = { enable = true; };
              };
              settings = {
                global.excludes = [ "**/vendor/**" ];
                formatter.prettier = {
                  # TODO: Use settings.pluginSearchDirs
                  options = [ "--write" ] ++ (if pkgs.stdenv.isLinux then
                    [ "--plugin-search-dir=${prettier-solidity}/lib" ]
                  else
                    [ ]);
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
