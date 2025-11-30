{
  description = "Union is a trust-minimized, zero-knowledge bridging protocol, designed for censorship resistance, extremely high security and usage in decentralized finance.";
  inputs = {
    solc = {
      url = "github:hellwolf/solc.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/release-24.11";
    # Track a separate nixpkgs for JS/TS toolchains
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
    # Remove when lnav is updated on upstream nixpkgs
    nixpkgs-lnav.url = "github:cor/nixpkgs/lnav-v0.12.2-beta";
    nixpkgs-go-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
    process-compose.url = "github:F1bonacc1/process-compose";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    arion.url = "github:hercules-ci/arion/v0.2.2.0";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    foundry = {
      url = "github:shazow/foundry.nix/main";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
    env-utils = {
      url = "github:oceanlewis/env-utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    ibc-go = {
      url = "github:unionlabs/ibc-go-union?rev=bfabb646cf7384bd33ee672f51a0e1325f545c10";
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
    # for chains that require blst (babylon)
    blst = {
      url = "github:supranational/blst?rev=3dd0f804b1819e5d03fb22ca2e6fac105932043a";
      flake = false;
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
    wasmd = {
      url = "github:unionlabs/wasmd?rev=913c24df4e0a7a3d791d27fc95313d559e9428b6";
      flake = false;
    };
    nix-filter.url = "github:numtide/nix-filter?rev=3449dc925982ad46246cfc36469baf66e1b64f17";
    get-flake.url = "github:ursi/get-flake";
    stargaze = {
      url = "git+https://github.com/public-awesome/stargaze?ref=refs/tags/v15.2.0&submodules=1";
      flake = false;
    };
    osmosis = {
      url = "github:osmosis-labs/osmosis/v28.0.5";
      flake = false;
    };
    atomone = {
      url = "github:atomone-hub/atomone/v3.0.3";
      flake = false;
    };
    xion = {
      url = "github:burnt-labs/xion/v21.0.0";
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
      repo = "cometbft";
      ref = "v1.0.1-cometbls";
      flake = false;
    };
    cosmossdk = {
      type = "github";
      owner = "unionlabs";
      repo = "cosmos-sdk";
      ref = "v0.50.13-cometblsv1";
      flake = false;
    };

    ethereum-consensus-specs = {
      url = "https://github.com/ethereum/consensus-spec-tests/releases/download/v1.4.0/general.tar.gz";
      flake = false;
    };

    # uniond versions
    v1_0_0 = {
      url = "github:unionlabs/union/release/uniond/v1.0.0";
      flake = false;
    };
    v1_1_0 = {
      url = "github:unionlabs/union/release/uniond/v1.1.1";
      flake = false;
    };
    v1_2_0 = {
      url = "github:unionlabs/union/release/uniond/v1.2.3";
      flake = false;
    };
  };
  outputs =
    inputs@{
      self,
      nixpkgs,
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
    let
      getRepoMeta = this: {
        gitRev = if (builtins.hasAttr "rev" this) then this.rev else "dirty";
        gitShortRev = this.shortRev or (this.dirtyShortRev or "dirty");
        lastModified = if this ? lastModified then builtins.toString this.lastModified else "0";
        lastModifiedDate = this.lastModifiedDate or "1970-01-01T00:00:00Z";
      };
    in
    with (getRepoMeta self);
    flake-parts.lib.mkFlake { inherit inputs; } {
      flake = {
        site = {
          x86_64-linux = {
            inherit (self.packages.x86_64-linux) site;
            inherit (self.packages.x86_64-linux) app2;
            inherit (self.packages.x86_64-linux) ceremony;
          };
          aarch64-linux = {
            inherit (self.packages.aarch64-linux) site;
            inherit (self.packages.aarch64-linux) app2;
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
        ./deployments
        ./uniond/uniond.nix
        ./galoisd/galoisd.nix
        ./unionvisor/unionvisor.nix
        ./voyager/voyager.nix
        ./mpc/mpc.nix
        ./lib/aptos.nix
        ./uniond/proto.nix
        ./docs/docs.nix
        ./docs/openapi.nix
        ./ceremony/ceremony.nix
        ./site/site.nix
        ./app2/app2.nix
        ./ts-sdk/ts-sdk.nix
        ./ts-sdk-evm/ts-sdk-evm.nix
        ./ts-sdk-cosmos/ts-sdk-cosmos.nix
        ./ts-sdk-sui/ts-sdk-sui.nix
        ./effect-svelte/effect-svelte.nix
        ./typescript-sdk/typescript-sdk.nix
        ./cosmwasm/cosmwasm.nix
        ./evm/evm.nix
        ./tools/rust-proto.nix
        ./tools/tools.nix
        ./tools/scarb.nix
        ./tools/wasm-light-client.nix
        ./tools/libwasmvm/libwasmvm.nix
        ./tools/libblst/libblst.nix
        ./tools/tidy/tidy.nix
        ./tools/rust/rust.nix
        ./tools/rust/crane.nix
        ./tools/tera/tera.nix
        ./tools/docgen/docgen.nix
        ./tools/todo-comment.nix
        ./tools/iaviewer/iaviewer.nix
        ./networks/e2e-setup.nix
        ./networks/devnet.nix
        ./networks/simulation/simd.nix
        ./networks/stargaze.nix
        ./networks/osmosis.nix
        ./networks/atomone.nix
        ./networks/xion.nix
        ./networks/babylon.nix
        ./networks/stride.nix
        ./e2e/all-tests.nix
        ./e2e/e2e.nix
        ./devnet-compose/devnet-compose.nix
        ./drip/drip.nix
        ./zkgm-dev/zkgm-dev.nix
        ./sentinel2/sentinel.nix
        ./lib/embed-commit
        ./networks/services/voyager.nix
        ./tools/union-test/tests/e2e.nix
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
            complete = versions.union-1.versions;
            first = pkgs.lib.lists.head complete;
            last = pkgs.lib.lists.last complete;
          };

          pkgsUnstable = import inputs.nixpkgs-unstable { inherit system; };
          pkgsGoUnstable = import inputs.nixpkgs-go-unstable { inherit system; };
          pnpm = pkgsUnstable.pnpm_10;
          nodejs = pkgsUnstable.nodejs-slim_24;
        in
        {
          _module = {
            args = {
              inherit
                gitRev
                gitShortRev
                lastModified
                lastModifiedDate
                nixpkgs
                dbg
                get-flake
                uniondBundleVersions
                pkgsUnstable
                pkgsGoUnstable
                mkCi
                ;

              pkgs = nixpkgs.legacyPackages.${system}.appendOverlays (
                with inputs;
                [
                  solc.overlay
                  rust-overlay.overlays.default
                  foundry.overlay
                  (_: super: {
                    inherit (self'.packages) devnet-utils;
                    mkRootDrv =
                      name: subAttrs:
                      subAttrs
                      // (builtins.removeAttrs
                        (pkgs.writeShellApplication {
                          inherit name;
                          text = ''
                            echo "this object (${name}) only has the following subattributes:"

                            ${pkgs.lib.concatMapStringsSep "\n" (a: "echo ${a}") (builtins.attrNames subAttrs)}

                            exit 1
                          '';
                        })
                        # cleanup the tab completion a bit
                        [
                          "checkPhase"
                          "doCheck"
                          "doInstallCheck"
                          "passthru"
                          "outputs"
                          "preferLocalBuild"
                          "propagatedBuildInputs"
                          "propagatedNativeBuildInputs"
                          "depsBuildBuild"
                          "depsBuildBuildPropagated"
                          "depsBuildTarget"
                          "depsBuildTargetPropagated"
                          "depsHostHost"
                          "depsHostHostPropagated"
                          "depsTargetTarget"
                          "depsTargetTargetPropagated"
                          "buildInputs"
                          "nativeBuildInputs"
                          "enableParallelBuilding"
                          "enableParallelChecking"
                          "enableParallelInstalling"
                          "cmakeFlags"
                          "configureFlags"
                          "__ignoreNulls"
                          "__structuredAttrs"
                          "allowSubstitutes"
                          "overrideAttrs"
                          "drvAttrs"
                          "meta"
                          "mesonFlags"
                          "passAsFile"
                          "patches"
                          "strictDeps"
                          "inputDerivation"
                          "executable"
                          "userHook"
                          "stdenv"
                          "all"
                          "out"
                          "buildCommand"
                          "args"
                          "builder"
                        ]
                      );

                    writeShellApplicationWithArgs = import ./tools/writeShellApplicationWithArgs.nix {
                      pkgs = super;
                    };

                    foundry-bin = super.foundry-bin.overrideAttrs (old: {
                      installPhase =
                        old.installPhase
                        + ''
                          # LD_LIBRARY_PATH must be set in the outer environment that cast is called in since it shells out to an auto-downloaded solc that it then attempts to patch for nixos, which, to everyone's surprise, does not work
                          mv $out/bin/cast $out/bin/cast-cursed

                          cat <<EOF >> $out/bin/cast
                          export LD_LIBRARY_PATH=${
                            lib.makeLibraryPath [
                              super.gcc14.cc.lib
                              # super.stdenv.cc.cc.lib
                            ]
                          }
                          $out/bin/cast-cursed "\$@"
                          unset LD_LIBRARY_PATH
                          EOF

                          echo "patched cast"

                          chmod +x $out/bin/cast

                          mv $out/bin/forge $out/bin/forge-cursed

                          cat <<EOF >> $out/bin/forge
                          export LD_LIBRARY_PATH=${
                            lib.makeLibraryPath [
                              super.gcc14
                              # super.stdenv.cc.cc.lib
                            ]
                          }
                          $out/bin/forge-cursed "\$@"
                          unset LD_LIBRARY_PATH
                          EOF

                          chmod +x $out/bin/forge

                          echo "patched forge"
                        '';
                    });

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
                            mainProgram = "solc";
                            license = super.lib.licenses.gpl3;
                          };
                        }
                      else
                        super.solc_0_8_27;
                  })
                ]
              );

              buildPnpmPackage = import ./tools/typescript/buildPnpmPackage.nix {
                inherit pnpm nodejs;
                pkgs = pkgsUnstable;
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
                inherit (inputs) interchain-security;
              };

              openapi = {
                uniondOpenApiYml = ./uniond/docs/static/openapi.yml;
                cometblsOpenApiYml = "${inputs.cometbls}/rpc/openapi/openapi.yaml";
                ibcGoOpenApiYml = "${inputs.ibc-go}/docs/client/swagger-ui/swagger.yaml";
              };
            };
          };

          packages = {
            default = mkCi false self'.packages.uniond;
            inherit (pkgs) solc;
            # lib.mkCrane = import ./tools/rust/mkCrane.nix;
            # sourceInfo = builtins.toFile "gitRev" (
            #   builtins.toJSON (
            #     builtins.removeAttrs self.sourceInfo [
            #       "narHash"
            #       "outPath"
            #     ]
            #   )
            # );
          };

          checks = {
            spellcheck = pkgs.stdenv.mkDerivation {
              name = "spellcheck";
              dontUnpack = true;
              src = ./.;
              buildInputs = [ pkgsUnstable.typos ];
              doCheck = true;
              checkPhase = ''
                cd $src/.
                typos --config=typos.toml --format=brief
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
              [
                rust.toolchains.dev
                nodejs
              ]
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
                xh
                self'.packages.tdc
                yq
                perl
                strace
              ])
              ++ [
                pkgsUnstable.wasm-bindgen-cli_0_2_100
                pkgs.twiggy
                pkgs.wasm-pack
              ]
              ++ (with pkgsUnstable; [
                bun
                pnpm_10
                cargo-machete
                deno
                nixd
                procs
                graphviz
                emmet-language-server
                typescript-go
                package-version-server
                nodePackages_latest.graphqurl
                nodePackages_latest.svelte-language-server
                nodePackages_latest."@astrojs/language-server"
                nodePackages_latest."@tailwindcss/language-server"
                nodePackages_latest.typescript-language-server
                nodePackages_latest.vscode-langservers-extracted
                binaryen
              ])
              ++ (with pkgs; [
                wasm-tools
                postgresql
                go_1_23
                gopls
                go-tools
                gotools
              ])
              ++ (
                if pkgs.stdenv.isLinux then
                  [
                    pkgs.solc
                    pkgs.foundry-bin
                    pkgs.sqlx-cli
                    self'.packages.ignite-cli
                    self'.packages.scarb
                    self'.packages.snforge
                    self'.packages.sncast
                    self'.packages.cairo-language-server
                    self'.packages.cairo-format
                    self'.packages.universal-sierra-compiler
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
            PROTOC = "${pkgs.protobuf}/bin/protoc";
            FOUNDRY_LIBS = ''["${self'.packages.evm-libs}"]'';
            FOUNDRY_DISABLE_NIGHTLY_WARNING = "1";
          };
          # https://flake.parts/options/treefmt-nix#opt-perSystem.treefmt
          treefmt = import ./treefmt.nix {
            inherit (self'.packages) movefmt;
            inherit
              lib
              pkgs
              pkgsUnstable
              rust
              ;
          };
        };
    }
    // {
      lib = {
        inherit getRepoMeta;
      };
    };

  nixConfig = {
    extra-substituters = [
      # "https://union.cachix.org/"
      "https://cache.garnix.io"
    ];
    extra-trusted-public-keys = [
      # "union.cachix.org-1:TV9o8jexzNVbM1VNBOq9fu8NK+hL6ZhOyOh0quATy+M="
      "cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g="
    ];
  };
}
