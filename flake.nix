{
  description = "Union: A trust-minimized, zero-knowledge bridging protocol for DeFi, designed for censorship resistance and high security.";

  # Organize inputs into logical groups for clarity
  inputs = {
    blockchain = {
      solc = {
        url = "github:hellwolf/solc.nix";
        inputs.nixpkgs.follows = "nixpkgs";
        # Solidity compiler for Ethereum smart contracts
      };
      foundry = {
        url = "github:shazow/foundry.nix/main";
        inputs.nixpkgs.follows = "nixpkgs";
        # Ethereum development toolkit
      };
      ibc-go = {
        url = "github:unionlabs/ibc-go-union?rev=bfabb646cf7384bd33ee672f51a0e1325f545c10";
        flake = false;
        # Inter-Blockchain Communication protocol
      };
      wasmd = {
        url = "github:unionlabs/wasmd?rev=913c24df4e0a7a3d791d27fc95313d559e9428b6";
        flake = false;
        # CosmWasm runtime for smart contracts
      };
      cosmosproto = {
        url = "github:cosmos/cosmos-proto?ref=refs/tags/v1.0.0-beta.5";
        flake = false;
      };
      gogoproto = {
        url = "github:cosmos/gogoproto?rev=34f37065b54523d08d7b637c78333d444f350e21";
        flake = false;
      };
      googleapis = {
        url = "github:googleapis/googleapis?rev=8984ddb508dea0e673b724c58338e810b1d8aee3";
        flake = false;
      };
      ics23 = {
        url = "github:cosmos/ics23";
        flake = false;
      };
      interchain-security = {
        url = "github:cosmos/interchain-security";
        flake = false;
      };
      cometbls = {
        url = "github:unionlabs/cometbft?ref=v1.0.1-cometbls";
        flake = false;
      };
      cosmossdk = {
        url = "github:unionlabs/cosmos-sdk?ref=v0.50.13-cometblsv1";
        flake = false;
      };
      stargaze = {
        url = "git+https://github.com/public-awesome/stargaze?ref=refs/tags/v15.2.0&submodules=1";
        flake = false;
      };
      osmosis = {
        url = "github:osmosis-labs/osmosis/v28.0.5";
        flake = false;
      };
      babylon = {
        url = "github:babylonlabs-io/babylon/v1.0.0-rc.7";
        flake = false;
      };
      stride = {
        url = "github:Stride-Labs/stride/v26.0.3";
        flake = false;
      };
      public-awesome-launchpad = {
        url = "github:public-awesome/launchpad/a14454cd2ee914af5ce10cd4cc94a9b6bfe660f6";
        flake = false;
      };
      cosmwasm-nfts = {
        url = "github:hussein-aitlahcen/cw-nfts/f2d7a07df63504ff8cbf0aad4140e56b3f5bfc3e";
        flake = false;
      };
      blst = {
        url = "github:supranational/blst?rev=3dd0f804b1819e5d03fb22ca2e6fac105932043a";
        flake = false;
        # For Babylon chain
      };
      ethereum-consensus-specs = {
        url = "https://github.com/ethereum/consensus-spec-tests/releases/download/v1.4.0/general.tar.gz";
        flake = false;
      };
    };
    tools = {
      nixpkgs.url = "github:NixOS/nixpkgs/release-24.11";
      nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
      nixpkgs-lnav.url = "github:cor/nixpkgs/lnav-v0.12.2-beta";
      flake-parts = {
        url = "github:hercules-ci/flake-parts";
        inputs.nixpkgs-lib.follows = "nixpkgs";
      };
      arion.url = "github:hercules-ci/arion/v0.2.2.0";
      treefmt-nix.url = "github:numtide/treefmt-nix";
      rust-overlay.url = "github:oxalica/rust-overlay";
      crane.url = "github:ipetkov/crane";
      env-utils = {
        url = "github:oceanlewis/env-utils";
        inputs.nixpkgs.follows = "nixpkgs";
      };
      nix-filter.url = "github:numtide/nix-filter?rev=3449dc925982ad46246cfc36469baf66e1b64f17";
      get-flake.url = "github:ursi/get-flake";
    };
    uniond-versions = {
      v1_0_0 = {
        url = "github:unionlabs/union/release/uniond/v1.0.0";
        flake = false;
      };
      v1_1_0 = {
        url = "github:unionlabs/union/release/uniond/v1.1.1";
        flake = false;
      };
      v1_2_0 = {
        url = "github:unionlabs/union/release/uniond/v1.2.0";
        flake = false;
      };
    };
  };

  outputs = inputs@{ self, nixpkgs, flake-parts, nix-filter, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];

      imports = [
        ./deployments
        ./uniond/uniond.nix
        ./galoisd/galoisd.nix
        ./unionvisor/unionvisor.nix
        ./voyager/voyager.nix
        ./mpc/mpc.nix
        ./hubble/hubble.nix
        ./lib/aptos.nix
        ./uniond/proto.nix
        ./docs/docs.nix
        ./docs/openapi.nix
        ./ceremony/ceremony.nix
        ./site/site.nix
        ./app2/app2.nix
        ./ts-sdk/ts-sdk.nix
        ./typescript-sdk/typescript-sdk.nix
        ./cosmwasm/cosmwasm.nix
        ./evm/evm.nix
        ./tools/rust-proto.nix
        ./tools/tools.nix
        ./tools/wasm-light-client.nix
        ./tools/libwasmvm/libwasmvm.nix
        ./tools/libblst/libblst.nix
        ./tools/tidy/tidy.nix
        ./tools/rust/rust.nix
        ./tools/rust/crane.nix
        ./tools/tera/tera.nix
        ./tools/docgen/docgen.nix
        ./tools/hasura-cli/hasura-cli.nix
        ./tools/todo-comment.nix
        ./tools/iaviewer/iaviewer.nix
        ./networks/e2e-setup.nix
        ./networks/devnet.nix
        ./networks/simulation/simd.nix
        ./networks/stargaze.nix
        ./networks/osmosis.nix
        ./networks/babylon.nix
        ./networks/stride.nix
        ./e2e/all-tests.nix
        ./e2e/e2e.nix
        ./devnet-compose/devnet-compose.nix
        ./drip/drip.nix
        ./zkgm-dev/zkgm-dev.nix
        ./sentinel/sentinel.nix
        ./sentinel2/sentinel.nix
        ./lib/embed-commit
        ./networks/services/voyager.nix
        inputs.tools.treefmt-nix.flakeModule
      ];

      perSystem = { config, self', pkgs, rust, system, lib, ... }:
        let
          mkCi = import ./tools/mkCi.nix { inherit pkgs; };
          pkgsUnstable = import inputs.tools.nixpkgs-unstable { inherit system; };
          gitRev = self.rev or "dirty";
          gitShortRev = self.shortRev or "dirty";
          lastModified = self.lastModified or "0";
          lastModifiedDate = self.lastModifiedDate or "1970-01-01T00:00:00Z";

          # Define overlays
          overlays = {
            solc = _: super: {
              solc = if system == "aarch64-linux" then
                super.gccStdenv.mkDerivation {
                  pname = "solc";
                  version = "0.8.27";
                  src = super.fetchurl {
                    url = "https://github.com/nikitastupin/solc/raw/main/linux/aarch64/solc-v${version}";
                    hash = "sha256-L5W7foccAyGJmcvINqByiDMJUYPuy0AOaVWKDvahCac=";
                  };
                  dontUnpack = true;
                  nativeBuildInputs = [ super.stdenv.cc.cc.lib super.autoPatchelfHook ];
                  installPhase = ''
                    runHook preInstall
                    mkdir -p $out/bin
                    cp ${src} $out/bin/solc
                    chmod +x $out/bin/solc
                    runHook postInstall
                  '';
                  meta = {
                    description = "Static binary of Solidity compiler";
                    homepage = "https://github.com/ethereum/solidity";
                    license = super.lib.licenses.gpl3;
                  };
                }
              else
                super.solc_0_8_27;
            };
            foundry-bin = _: super: {
              foundry-bin = super.foundry-bin.overrideAttrs (old: {
                installPhase = old.installPhase + ''
                  for bin in cast forge; do
                    mv $out/bin/$bin $out/bin/$bin-cursed
                    cat <<EOF > $out/bin/$bin
                    export LD_LIBRARY_PATH=${super.lib.makeLibraryPath [ super.stdenv.cc.cc.lib ]}
                    $out/bin/$bin-cursed "\$@"
                    unset LD_LIBRARY_PATH
                    EOF
                    chmod +x $out/bin/$bin
                  done
                '';
              });
            };
          };
        in
        {
          _module.args = {
            inherit gitRev gitShortRev lastModified lastModifiedDate nixpkgs pkgsUnstable mkCi;
            pkgs = nixpkgs.legacyPackages.${system}.appendOverlays (
              with inputs; [
                blockchain.solc.overlay
                tools.rust-overlay.overlays.default
                blockchain.foundry.overlay
                overlays.solc
                overlays.foundry-bin
              ]
            );
            ensureAtRepositoryRoot = ''
              if [[ -f flake.nix ]]; then
                echo "At repository root. Running script..."
              else
                echo "Not at repository root. Please cd to the root and try again."
                exit 1
              fi
            '';
            devnetConfig = {
              validatorCount = 4;
              ethereum.beacon.validatorCount = 128;
            };
            nix-filter = nix-filter.lib;
            proto = {
              inherit (inputs.blockchain) wasmd ibc-go ics23 cosmosproto gogoproto googleapis cometbls cosmossdk interchain-security;
              uniond = ./uniond/proto;
              galoisd = ./galoisd/proto;
              cometbls-lc = ./11-cometbls/proto;
            };
            openapi = {
              uniondOpenApiYml = ./uniond/docs/static/openapi.yml;
              cometblsOpenApiYml = "${inputs.blockchain.cometbls}/rpc/openapi/openapi.yaml";
              ibcGoOpenApiYml = "${inputs.blockchain.ibc-go}/docs/client/swagger-ui/swagger.yaml";
            };
            cw-instantiate2-salt = "61616161";
          };

          packages = {
            default = mkCi false self'.packages.uniond;
            inherit (pkgs) solc;
            union-cli = pkgs.writeShellApplication {
              name = "union-cli";
              text = ''
                ${self._module.args.ensureAtRepositoryRoot}
                case $1 in
                  devnet)
                    echo "Starting devnet..."
                    nix run .#devnet-compose
                    ;;
                  test)
                    echo "Running tests..."
                    cargo test
                    ;;
                  docs)
                    echo "Generating docs..."
                    nix build .#docs
                    ;;
                  *)
                    echo "Usage: union-cli [devnet|test|docs]"
                    exit 1
                    ;;
                esac
              '';
              buildInputs = [ pkgs.cargo ];
            };
            dependency-graph = pkgs.writeShellApplication {
              name = "dependency-graph";
              text = ''
                nix-tree .#default > deps.dot
                dot -Tpng deps.dot -o deps.png
                echo "Dependency graph generated at deps.png"
              '';
              buildInputs = [ pkgs.nix-tree pkgs.graphviz ];
            };
            docs = pkgs.stdenv.mkDerivation {
              name = "union-docs";
              src = nix-filter.lib.filter {
                root = ./docs;
                exclude = [ "node_modules" ".git" ];
              };
              buildInputs = [ pkgsUnstable.mdbook ];
              buildPhase = ''
                mdbook build
              '';
              installPhase = ''
                mv book $out
              '';
            };
          };

          checks = {
            spellcheck = pkgs.runCommand "spellcheck" {
              buildInputs = [ pkgsUnstable.typos ];
            } ''
              typos --config=${./typos.toml} --format=brief ${./.}
              touch $out
            '';
            nix-lint = pkgs.runCommand "nix-lint" {
              buildInputs = [ pkgs.statix ];
            } ''
              statix check ${./.}
              touch $out
            '';
            nil = mkCi (system == "x86_64-linux") (
              pkgs.stdenv.mkDerivation {
                name = "nil";
                src = nix-filter.lib.filter {
                  root = ./.;
                  exclude = [ "vendor" "node_modules" ".git" ];
                };
                buildInputs = [ pkgs.nil ];
                doCheck = true;
                checkPhase = ''
                  for i in $(find . -name "*.nix" -type f); do
                    nil diagnostics "$i"
                  done
                  touch $out
                '';
              }
            );
          };

          devShells = {
            default = pkgs.mkShell {
              name = "union-devShell";
              buildInputs = with pkgs; [
                rust.toolchains.dev
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
                xh
                self'.packages.tdc
                yq
                perl
                wasm-tools
                postgresql
                go_1_23
                gopls
                go-tools
                gotools
              ] ++ lib.optionals stdenv.isLinux [
                solc
                foundry-bin
                sqlx-cli
                self'.packages.hasura-cli
                self'.packages.ignite-cli
              ];
              nativeBuildInputs = [ config.treefmt.build.wrapper ] ++ lib.attrValues config.treefmt.build.programs;
              shellHook = ''
                ${self._module.args.ensureAtRepositoryRoot}
                export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath [ pkgs.stdenv.cc.cc.lib ]}
                export GOPRIVATE="github.com/unionlabs/*"
                export PUPPETEER_SKIP_DOWNLOAD=1
                export NODE_OPTIONS="--no-warnings"
                export ASTRO_TELEMETRY_DISABLED=1
                export ICS23_TEST_SUITE_DATA_DIR="${inputs.blockchain.ics23}/testdata"
                export ETHEREUM_CONSENSUS_SPECS_DIR="${inputs.blockchain.ethereum-consensus-specs}"
                export RUST_SRC_PATH="${rust.toolchains.dev}/lib/rustlib/src/rust/library"
                export VOYAGER_CONFIG_FILE_PATH="voyager/devnet-config.json"
                export SQLX_OFFLINE=true
                export LIBCLANG_PATH="${pkgs.llvmPackages_14.libclang.lib}/lib"
                export PROTOC="${pkgs.protobuf}/bin/protoc"
                export FOUNDRY_LIBS='["${self'.packages.evm-libs}"]'
                export FOUNDRY_DISABLE_NIGHTLY_WARNING="1"
                echo "Union development environment loaded."
              '';
            };
            frontend = pkgs.mkShell {
              name = "union-frontend-devShell";
              buildInputs = with pkgsUnstable; [
                bun
                pnpm_10
                deno
                nixd
                procs
                emmet-language-server
                nodePackages_latest.nodejs
                nodePackages_latest.graphqurl
                nodePackages_latest.svelte-language-server
                nodePackages_latest."@astrojs/language-server"
                nodePackages_latest."@tailwindcss/language-server"
                nodePackages_latest.typescript-language-server
                nodePackages_latest.vscode-langservers-extracted
              ];
              shellHook = ''
                echo "Frontend development environment loaded."
              '';
            };
          };

          treefmt = import ./treefmt.nix {
            inherit (self'.packages) movefmt;
            inherit lib pkgs pkgsUnstable rust;
          };
        };
    };

  nixConfig = {
    extra-substituters = [
      "https://cache.garnix.io"
    ];
    extra-trusted-public-keys = [
      "cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g="
    ];
  };
}
