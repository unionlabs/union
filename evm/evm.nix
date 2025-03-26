_: {
  perSystem =
    {
      self',
      pkgs,
      proto,
      nix-filter,
      ensureAtRepositoryRoot,
      system,
      mkCi,
      gitRev,
      ...
    }:
    let
      solidity-stringutils = pkgs.fetchFromGitHub {
        owner = "Arachnid";
        repo = "solidity-stringutils";
        rev = "4b2fcc43fa0426e19ce88b1f1ec16f5903a2e461";
        hash = "sha256-Hwc6akOane0feJw7xW+pbT4KsHVOb8JFMhc61F7sej4=";
      };
      solidity-bytes-utils = pkgs.fetchFromGitHub {
        owner = "GNSPS";
        repo = "solidity-bytes-utils";
        rev = "v0.8.2";
        hash = "sha256-eDAYc7qoBR/nW9hKBwO0VcpAG+AYkxNWArqaXZwAL+Y=";
      };
      solady = pkgs.fetchFromGitHub {
        owner = "vectorized";
        repo = "solady";
        rev = "v0.1.12";
        hash = "sha256-XsIXs3lj5gddBzswNFY1DhnlhUQx+ITf6lvBPSkMY7c=";
      };
      forge-std = pkgs.fetchFromGitHub {
        owner = "foundry-rs";
        repo = "forge-std";
        rev = "v1.9.6";
        hash = "sha256-4y1Hf0Te2oJxwKBOgVBEHZeKYt7hs+wTgdIO+rItj0E=";
        fetchSubmodules = true;
      };
      openzeppelin = pkgs.fetchFromGitHub {
        owner = "OpenZeppelin";
        repo = "openzeppelin-contracts";
        rev = "v5.3.0-rc.0";
        hash = "sha256-rCuoPQpHgJ7MjoJ9tNmL/YpW2d6EB+QM3nv6E8X3GV0=";
      };
      openzeppelin-upgradeable = pkgs.fetchFromGitHub {
        owner = "OpenZeppelin";
        repo = "openzeppelin-contracts-upgradeable";
        rev = "v5.2.0";
        hash = "sha256-AKPTlbGkIPK7yYQJH9cEdvHSF5ZM5hFWmaxtEkMhoxQ=";
      };
      openzeppelin-foundry-upgrades = pkgs.fetchFromGitHub {
        owner = "OpenZeppelin";
        repo = "openzeppelin-foundry-upgrades";
        rev = "v0.4.0";
        hash = "sha256-e9hnHibo0HXr+shOS6tNEOTu65DyCpwP0DjPRznqMxU=";
      };
      libraries = pkgs.linkFarm "evm-libraries" [
        {
          name = "solidity-stringutils";
          path = "${solidity-stringutils}";
        }
        {
          name = "solidity-bytes-utils";
          path = "${solidity-bytes-utils}";
        }
        {
          name = "solady";
          path = "${solady}/src";
        }
        {
          name = "forge-std";
          path = "${forge-std}/src";
        }
        {
          name = "@openzeppelin";
          path = "${openzeppelin}";
        }
        {
          name = "@openzeppelin-upgradeable";
          path = "${openzeppelin-upgradeable}";
        }
        {
          name = "@openzeppelin-foundry-upgradeable";
          path = "${openzeppelin-foundry-upgrades}/src";
        }
      ];
      evm-libs = pkgs.stdenv.mkDerivation {
        name = "evm-libs-src";
        phases = [
          "installPhase"
          "fixupPhase"
        ];
        src = libraries;
        installPhase = ''
          mkdir -p $out
          cp -rL $src/* $out
        '';
      };
      evmSources = pkgs.stdenv.mkDerivation {
        name = "evm-union-src";
        phases = [
          "installPhase"
          "fixupPhase"
        ];
        src = evm-libs;
        installPhase = ''
          mkdir -p $out/libs
          cp -rL $src/* $out/libs
          cp -r ${
            nix-filter {
              root = ./.;
              include = [
                "scripts"
                "contracts"
                "tests"
              ];
            }
          }/* $out/
        '';
        fixupPhase = ''
          substitute $out/contracts/internal/Versioned.sol $out/contracts/internal/Versioned.sol \
              --replace-fail 'dirty' '${gitRev}'
        '';
      };
      # Foundry FS permissions must be explicitly set in the config file
      foundryConfig = pkgs.writeTextDir "/foundry.toml" ''
        [profile.default.optimizer_details]
        cse = true
        constantOptimizer = true
        yul = true

        [profile.default]
        fs_permissions = [{ access = "read", path = "./" }, { access = "write", path = "contracts.json" }]
        libs = ["libs"]
        gas_reports = ["*"]
        via_ir = true
        ast = true
        optimizer = true
        optimizer_runs = 1_000

        [profile.script]
        src = "scripts"

        [profile.test]
        test = "tests/src"
      '';
      compilers = pkgs.linkFarm "evm-libraries" [
        {
          name = ".svm/${pkgs.solc.version}/solc-${pkgs.solc.version}";
          path = "${pkgs.lib.getExe pkgs.solc}";
        }
      ];
      wrappedForge = pkgs.symlinkJoin {
        name = "forge";
        paths = [ pkgs.foundry-bin ];
        buildInputs = [ pkgs.makeWrapper ];
        postBuild = ''
          wrapProgram $out/bin/forge \
            --append-flags "--offline --no-auto-detect" \
            --set HOME ${compilers} \
            --set SSL_CERT_FILE "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" \
            --set FOUNDRY_CONFIG "${foundryConfig}/foundry.toml"
        '';
      };
      wrappedForgeOnline = pkgs.symlinkJoin {
        name = "forge";
        paths = [ pkgs.foundry-bin ];
        buildInputs = [ pkgs.makeWrapper ];
        postBuild = ''
          wrapProgram $out/bin/forge \
            --set HOME ${compilers} \
            --set SSL_CERT_FILE "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" \
            --set FOUNDRY_CONFIG "${foundryConfig}/foundry.toml"
        '';
      };
      networks = [
        {
          network = "devnet";
          rpc-url = "http://localhost:8545";
          private-key = "0x${builtins.readFile ./../networks/genesis/devnet-eth/dev-key0.prv}";
          extra-args = pkgs.lib.optionalString pkgs.stdenv.isx86_64 "--verify --verifier blockscout --verifier-url http://localhost/api";
        }
        {
          # for use with the local berachain devnet from berachain/beacon-kit
          network = "berachain-devnet";
          rpc-url = "http://localhost:8545";
          private-key = "0xfffdbb37105441e14b0ee6330d855d8504ff39e705c3afa8f859ac9865f99306";
        }
        {
          # for use with the local arbitrum devnet from offchainlabs/nitro-testnode
          network = "arbitrum-devnet";
          rpc-url = "http://localhost:8547";
          private-key = "0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659";
        }
        {
          network = "sepolia";
          rpc-url = "https://0xrpc.io/sep";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key)"'';
          extra-args = ''--verify --verifier etherscan --etherscan-api-key "$1"'';
        }
        {
          network = "holesky";
          rpc-url = "https://1rpc.io/holesky";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key)"'';
          extra-args = ''--verify --verifier etherscan --etherscan-api-key "$1"'';
        }
        {
          network = "0g-testnet";
          rpc-url = "https://evmrpc-testnet.0g.ai";
          private-key = ''"$1"'';
          extra-args = " --legacy --batch-size=1";
          # extra-args = ''--verify --verifier etherscan --etherscan-api-key "$2"'';
        }
        {
          network = "scroll-testnet";
          rpc-url = "https://sepolia-rpc.scroll.io";
          private-key = ''"$1"'';
          extra-args = ''--verify --verifier etherscan --verifier-url https://api-sepolia.scrollscan.com/api --etherscan-api-key "$2"'';
        }
        {
          network = "arbitrum-testnet";
          rpc-url = "https://sepolia-rollup.arbitrum.io/rpc";
          private-key = ''"$1"'';
        }
        {
          network = "berachain-testnet";
          rpc-url = "https://fabled-serene-mountain.bera-bartio.quiknode.pro/6ab3f499dcce3d52591ce97a5f07a13fae75deb1/";
          private-key = ''"$1"'';
        }
      ];

      eth-deploy =
        {
          rpc-url,
          private-key,
          extra-args ? "",
          ...
        }:
        mkCi false (
          pkgs.writeShellApplication {
            name = "eth-deploy-ibc";
            runtimeInputs = [ self'.packages.forge ];
            text = ''
              ${ensureAtRepositoryRoot}
              OUT="$(mktemp -d)"
              pushd "$OUT"
              cp --no-preserve=mode -r ${self'.packages.evm-contracts}/* .
              cp --no-preserve=mode -r ${evmSources}/* .

              PRIVATE_KEY=${private-key} \
              DEPLOYER="$3" \
              FOUNDRY_PROFILE="script" \
                forge script scripts/Deploy.s.sol:DeployIBC \
                -vvvv \
                --rpc-url ${rpc-url} \
                --broadcast ${extra-args}

              popd
              rm -rf "$OUT"
            '';
          }
        );

      get-deployed-heights =
        { rpc-url, ... }:
        pkgs.writeShellApplication {
          name = "get-deployed-heights";
          runtimeInputs = [
            self'.packages.forge
            pkgs.moreutils
          ];
          runtimeEnv = {
            ETH_RPC_URL = rpc-url;
          };
          text = ''
            ${ensureAtRepositoryRoot}

            DEPLOYMENTS_FILE="deployments/deployments-testnet-10.json"
            export DEPLOYMENTS_FILE

            CHAIN_ID="$(cast chain-id)"
            export CHAIN_ID

            echo "chain id: $CHAIN_ID"

            jq \
              '. |= map(if .chain_id == $chain_id then .deployments.core.height = ($height | tonumber) else . end)' \
              "$DEPLOYMENTS_FILE" \
              --arg chain_id "$CHAIN_ID" \
              --arg height "$(( "$(
                cast logs 'Initialized(uint64)' \
                  --address "$(
                    cast impl "$(
                        jq -r \
                          '.[] | select(.chain_id == $chain_id) | .deployments.core.address' \
                          "$DEPLOYMENTS_FILE" \
                          --arg chain_id "$CHAIN_ID"
                      )"
                  )" \
                  --json \
                | jq -r '.[0].blockNumber'
              )" ))" \
            | sponge "$DEPLOYMENTS_FILE"

            for key in lightclient app ; do
              echo "key: $key"
              jq -r \
                '.[] | select(.chain_id == $chain_id) | .deployments[$key] | keys[]' \
                "$DEPLOYMENTS_FILE" \
                --arg chain_id "$CHAIN_ID" \
                --arg key "$key" \
                | while read -r subkey ; do
                  echo "$key: $subkey"
                  jq \
                    '. |= map(if .chain_id == $chain_id then .deployments[$key][$subkey].height = ($height | tonumber) else . end)' \
                    "$DEPLOYMENTS_FILE" \
                    --arg chain_id "$CHAIN_ID" \
                    --arg subkey "$subkey" \
                    --arg key "$key" \
                    --arg height "$(( "$(
                      cast logs 'Initialized(uint64)' \
                        --address "$(
                          cast impl "$(
                              jq -r \
                                '.[] | select(.chain_id == $chain_id) | .deployments[$key][$subkey].address' \
                                "$DEPLOYMENTS_FILE" \
                                --arg chain_id "$CHAIN_ID" \
                                --arg subkey "$subkey" \
                                --arg key "$key"
                            )"
                        )" \
                        --json \
                      | jq -r '.[0].blockNumber'
                    )" ))" \
                  | sponge "$DEPLOYMENTS_FILE"
                done
            done
          '';
        };

      eth-deploy-full =
        {
          rpc-url,
          private-key,
          extra-args ? "",
          ...
        }:
        mkCi false (
          pkgs.writeShellApplication {
            name = "eth-deploy-full";
            runtimeInputs = [ self'.packages.forge ];
            text = ''
              ${ensureAtRepositoryRoot}
              OUT="$(mktemp -d)"
              pushd "$OUT"
              cp --no-preserve=mode -r ${self'.packages.evm-contracts}/* .
              cp --no-preserve=mode -r ${evmSources}/* .

              PRIVATE_KEY=${private-key} \
              FOUNDRY_LIBS='["libs"]' \
              FOUNDRY_PROFILE="script" \
                forge script scripts/Deploy.s.sol:DeployDeployerAndIBC \
                -vvvv \
                --rpc-url ${rpc-url} \
                --broadcast ${extra-args}

              popd
              rm -rf "$OUT"
            '';
          }
        );

      eth-verify =
        {
          rpc-url,
          private-key,
          extra-args ? "",
          ...
        }:
        mkCi false (
          pkgs.writeShellApplication {
            name = "eth-verify";
            runtimeInputs = [ wrappedForgeOnline ];
            text = ''
              ${ensureAtRepositoryRoot}
              nix run .#evm-contracts-addresses -- "$1" "$2" ${rpc-url}

              PROJECT_ROOT=$(pwd)
              OUT="$(mktemp -d)"
              pushd "$OUT"
              cp --no-preserve=mode -r ${self'.packages.evm-contracts}/* .
              cp --no-preserve=mode -r ${evmSources}/* .

              jq -r 'to_entries | map([.key, .value.args, .value.contract]) | .[] | @tsv' "$PROJECT_ROOT"/contracts.json | \
                while IFS=$'\t' read -r address args contract; do
                  if [ "$address" != "0x0000000000000000000000000000000000000000" ]
                  then
                    PRIVATE_KEY=${private-key} \
                    FOUNDRY_LIBS='["libs"]' \
                    FOUNDRY_PROFILE="script" \
                      forge verify-contract \
                        --force \
                        --watch "$address" "$contract" \
                        --constructor-args "$args" \
                        --api-key "$3" \
                        --rpc-url ${rpc-url} || true
                  fi
                done

              popd
              rm -rf "$OUT"
            '';
          }
        );

      eth-deploy-single =
        {
          rpc-url,
          kind,
          extra-args ? "",
          ...
        }:
        mkCi false (
          pkgs.writeShellApplicationWithArgs {
            name = "eth-deploy-single-${kind}";
            runtimeInputs = [ self'.packages.forge ];
            arguments = [
              {
                arg = "deployer_pk";
                required = true;
                help = "The deployer contract address.";
              }
              {
                arg = "private_key";
                required = true;
                help = "The contract owner private key.";
              }
              {
                arg = "sender_pk";
                required = true;
                help = "The sender address that created the contract through the deployer.";
              }
            ];
            text = ''
              ${ensureAtRepositoryRoot}
              OUT="$(mktemp -d)"
              pushd "$OUT"
              cp --no-preserve=mode -r ${self'.packages.evm-contracts}/* .
              cp --no-preserve=mode -r ${evmSources}/* .

              DEPLOYER="$argc_deployer_pk" \
              SENDER="$argc_sender_pk" \
              PRIVATE_KEY="$argc_private_key" \
              FOUNDRY_LIBS='["libs"]' \
              FOUNDRY_PROFILE="script" \
                forge script scripts/Deploy.s.sol:Deploy${kind} \
                -vvvv \
                --rpc-url "${rpc-url}" \
                --broadcast ${extra-args}

              popd
              rm -rf "$OUT"
            '';
          }
        );

      eth-upgrade =
        {
          dry ? false,
          rpc-url,
          protocol,
          private-key,
          ...
        }:
        mkCi false (
          pkgs.writeShellApplicationWithArgs {
            name = "evm-${pkgs.lib.optionalString dry "dry"}upgrade-${protocol}";
            runtimeInputs = [
              self'.packages.forge
              pkgs.jq
            ];
            arguments =
              [
                {
                  arg = "deployer_pk";
                  required = true;
                  help = "The deployer contract address.";
                }
                {
                  arg = "sender_pk";
                  required = true;
                  help = "The sender address that created the contract through the deployer.";
                }
              ]
              ++ pkgs.lib.optional dry {
                arg = "owner_pk";
                required = true;
                help = "The contract owner public key to prank.";
              };
            text = ''
              OUT="$(mktemp -d)"
              pushd "$OUT"
              cp --no-preserve=mode -r ${self'.packages.evm-contracts}/* .
              cp --no-preserve=mode -r ${evmSources}/* .

              DEPLOYER="$argc_deployer_pk" \
              SENDER="$argc_sender_pk" \
              OWNER="${pkgs.lib.optionalString dry "$argc_owner_pk"}" \
              PRIVATE_KEY=${private-key} \
              FOUNDRY_LIBS='["libs"]' \
              FOUNDRY_PROFILE="script" \
                forge script scripts/Deploy.s.sol:${pkgs.lib.optionalString dry "Dry"}Upgrade${protocol} -vvvvv \
                  --rpc-url ${rpc-url} \
                  --broadcast

              rm -rf "$OUT"
              popd
            '';
          }
        );
    in
    {
      packages = {
        # Beware, the generate solidity code is broken and require manual patch. Do not update unless you know that aliens exists.
        generate-sol-proto = mkCi false (
          pkgs.writeShellApplication {
            name = "generate-sol-proto";
            runtimeInputs = [ pkgs.protobuf ];
            text =
              let
                solidity-protobuf = pkgs.stdenv.mkDerivation {
                  name = "solidity-protobuf";
                  version = "0.0.1";
                  src = pkgs.fetchFromGitHub {
                    owner = "CyrusVorwald";
                    repo = "solidity-protobuf";
                    rev = "1c323bed92d373d6c4d6c728c8dd9f76cf4b5a0c";
                    hash = "sha256-1obEhMjaLToaSk920CiJwfhkw+LDgY5Y/b7SpkeuqDE=";
                  };
                  buildInputs = [
                    (pkgs.python3.withPackages (
                      ps: with ps; [
                        protobuf
                        wrapt
                      ]
                    ))
                  ];
                  buildPhase = "true";
                  installPhase = ''
                    mkdir $out
                    cp -r $src/* $out
                  '';
                };
                protoIncludes = ''-I"${proto.cometbls}/proto" -I"${proto.cosmossdk}/proto" -I"${proto.ibc-go}/proto" -I"${proto.cosmosproto}/proto" -I"${proto.ics23}/proto" -I"${proto.googleapis}" -I"${proto.gogoproto}" -I"${proto.uniond}"'';
              in
              ''
                plugindir="${solidity-protobuf}/protobuf-solidity/src/protoc"
                # find ${proto.ibc-go}/proto -name "$1" |\
                # while read -r file; do
                #   echo "Generating $file"
                #   protoc \
                #     ${protoIncludes} \
                #    -I"$plugindir/include" \
                #    --plugin="protoc-gen-sol=$plugindir/plugin/gen_sol.py" \
                #    --sol_out=gen_runtime="ProtoBufRuntime.sol&solc_version=0.8.21:$2" \
                #     "$file"
                # done
                # find ${proto.cometbls}/proto -type f -regex ".*canonical.proto" |\
                # while read -r file; do
                #   echo "Generating $file"
                #   protoc \
                #     ${protoIncludes} \
                #    -I"$plugindir/include" \
                #    --plugin="protoc-gen-sol=$plugindir/plugin/gen_sol.py" \
                #    --sol_out=gen_runtime="ProtoBufRuntime.sol&solc_version=0.8.21:$2" \
                #     "$file"
                # done

                find ${proto.uniond} -type f -regex ".*ibc.*cometbls.*proto" |\
                while read -r file; do
                  echo "Generating $file"
                  protoc \
                    ${protoIncludes} \
                   -I"$plugindir/include" \
                   --plugin="protoc-gen-sol=$plugindir/plugin/gen_sol.py" \
                   --sol_out=gen_runtime="ProtoBufRuntime.sol&solc_version=0.8.21:$2" \
                    "$file"
                done
              '';
          }
        );

        inherit evm-libs;

        evm-contracts = mkCi (system == "x86_64-linux") (
          pkgs.stdenv.mkDerivation {
            name = "evm-contracts";
            src = evmSources;
            buildInputs = [
              wrappedForge
              pkgs.solc
            ];
            buildPhase = ''
              forge --version
              FOUNDRY_PROFILE=script forge build --sizes
            '';
            doCheck = true;
            checkPhase = ''
              FOUNDRY_PROFILE=test forge test -vvv --out=tests-out --cache-path=tests-cache
            '';
            installPhase = ''
              mkdir -p $out
              mv out $out
              mv cache $out
            '';
          }
        );

        # Stack too deep :), again
        #
        # solidity-coverage =
        #   pkgs.runCommand "solidity-coverage"
        #     {
        #       buildInputs = [
        #         self'.packages.forge
        #         pkgs.lcov
        #       ];
        #     }
        #     ''
        #         cp --no-preserve=mode -r ${evmSources}/* .
        #         FOUNDRY_PROFILE="test" forge coverage --ir-minimum --report lcov
        #         lcov --remove ./lcov.info -o ./lcov.info.pruned \
        #           'contracts/Multicall.sol' \
        #           'contracts/apps/ucs/00-pingpong/*' \
        #           'contracts/lib/*' \
        #           'contracts/core/OwnableIBCHandler.sol' \
        #           'contracts/core/24-host/IBCCommitment.sol' \
        #           'contracts/core/25-handler/IBCHandler.sol' \
        #           'tests/*'
        #         genhtml lcov.info.pruned -o $out --branch-coverage
        #       mv lcov.info.pruned $out/lcov.info
        #     '';
        # show-solidity-coverage = pkgs.writeShellApplication {
        #   name = "show-solidity-coverage";
        #   runtimeInputs = [ ];
        #   text = ''
        #     xdg-open ${self'.packages.solidity-coverage}/index.html
        #   '';
        # };

        hubble-abis =
          let
            contracts = self'.packages.evm-contracts;
          in
          mkCi false (
            pkgs.runCommand "hubble-abis"
              {
                buildInputs = [ pkgs.jq ];
              }
              ''
                mkdir -p $out
                cd $out

                jq --compact-output --slurp 'map(.abi) | add' \
                  ${contracts}/out/OwnableIBCHandler.sol/OwnableIBCHandler.json > core.json

                jq --compact-output --slurp 'map(.abi) | add' \
                  ${contracts}/out/Zkgm.sol/ZkgmLib.json \
                  ${contracts}/out/Zkgm.sol/AbiExport.json \
                  ${contracts}/out/Zkgm.sol/UCS03Zkgm.json > app.ucs03.json

                jq --compact-output --slurp 'map(.abi) | add' \
                  ${contracts}/out/CometblsClient.sol/CometblsClient.json \
                  ${contracts}/out/CometblsClient.sol/CometblsClientLib.json > lightclient.cometbls.json

                jq --compact-output --slurp 'map(.abi) | add' \
                  ${contracts}/out/StateLensIcs23MptClient.sol/StateLensIcs23MptClient.json \
                  ${contracts}/out/StateLensIcs23MptClient.sol/StateLensIcs23MptLib.json > lightclient.state-lens-ics23-mpt.json

                jq --compact-output --slurp 'map(.abi) | add' \
                  ${contracts}/out/StateLensIcs23Ics23Client.sol/StateLensIcs23Ics23Client.json \
                  ${contracts}/out/StateLensIcs23Ics23Client.sol/StateLensIcs23Ics23Lib.json > lightclient.state-lens-ics23-ics23.json

                jq --compact-output --slurp 'map(.abi) | add' \
                  ${contracts}/out/StateLensIcs23SmtClient.sol/StateLensIcs23SmtClient.json \
                  ${contracts}/out/StateLensIcs23SmtClient.sol/StateLensIcs23SmtLib.json > lightclient.state-lens-ics23-smt.json
              ''
          );

        evm-contracts-addresses = mkCi false (
          pkgs.writeShellApplication {
            name = "evm-contracts-addresses";
            runtimeInputs = [
              self'.packages.forge
              pkgs.jq
            ];
            text = ''
              ${ensureAtRepositoryRoot}
              OUT="$(mktemp -d)"
              pushd "$OUT"
              cp --no-preserve=mode -r ${self'.packages.evm-contracts}/* .
              cp --no-preserve=mode -r ${evmSources}/* .

              DEPLOYER="$1" \
              SENDER="$2" \
              OUTPUT="contracts.json" \
              FOUNDRY_PROFILE="script" \
                  forge script scripts/Deploy.s.sol:GetDeployed -vvvv --fork-url "$3"

              popd
              cp "$OUT"/contracts.json contracts.json
              rm -rf "$OUT"
            '';
          }
        );

        forge = wrappedForge;

        evm-sources = evmSources;

        evm-deployer-image =
          let
            forge-deploy = pkgs.writeShellApplication {
              name = "forge-deploy";
              runtimeInputs = [ self'.packages.forge ];
              text = ''
                mkdir -p /evm
                cd /evm
                cp --no-preserve=mode -r ${self'.packages.evm-contracts}/* .
                cp --no-preserve=mode -r ${self'.packages.evm-sources}/* .
                FOUNDRY_PROFILE="script" forge script scripts/Deploy.s.sol:DeployDeployerAndIBC -vvv --rpc-url "$RPC_URL" --broadcast
              '';
            };
          in
          mkCi (system == "x86_64-linux") (
            pkgs.dockerTools.buildLayeredImage {
              name = "evm-deployer-image";
              contents = [
                pkgs.coreutils
                pkgs.curl
                pkgs.jq
                forge-deploy
                self'.packages.evm-sources
                self'.packages.evm-contracts
              ];
              config = {
                Entrypoint = [ (pkgs.lib.getExe forge-deploy) ];
                Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
              };
            }
          );
        eth-scripts =
          (derivation {
            name = "eth-scripts-empty-derivation-to-make-top-level-packages-happy";
          })
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "eth-verify-${args.network}";
              value = eth-verify args;
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "eth-deploy-${args.network}-full";
              value = eth-deploy-full args;
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "eth-deploy-${args.network}";
              value = eth-deploy args;
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "eth-deploy-${args.network}-state-lens-ics23-mpt-client";
              value = eth-deploy-single ({ kind = "StateLensIcs23MptClient"; } // args);
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "eth-deploy-${args.network}-state-lens-ics23-ics23-client";
              value = eth-deploy-single ({ kind = "StateLensIcs23Ics23Client"; } // args);
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "eth-deploy-${args.network}-state-lens-ics23-smt-client";
              value = eth-deploy-single ({ kind = "StateLensIcs23SmtClient"; } // args);
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "eth-deploy-${args.network}-ucs03";
              value = eth-deploy-single ({ kind = "UCS03"; } // args);
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "eth-deploy-${args.network}-multicall";
              value = eth-deploy-single ({ kind = "Multicall"; } // args);
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "eth-dryupgrade-${args.network}-ucs03";
              value = eth-upgrade (
                {
                  dry = true;
                  protocol = "UCS03";
                }
                // args
              );
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "eth-dryupgrade-${args.network}-cometbls-client";
              value = eth-upgrade (
                {
                  dry = true;
                  protocol = "CometblsClient";
                }
                // args
              );
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "eth-upgrade-${args.network}-ucs03";
              value = eth-upgrade (
                {
                  dry = false;
                  protocol = "UCS03";
                }
                // args
              );
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "eth-upgrade-${args.network}-state-lens-ics23-mpt-client";
              value = eth-upgrade (
                {
                  dry = false;
                  protocol = "StateLensIcs23MptClient";
                }
                // args
              );
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "eth-upgrade-${args.network}-state-lens-ics23-ics23-client";
              value = eth-upgrade (
                {
                  dry = false;
                  protocol = "StateLensIcs23Ics23Client";
                }
                // args
              );
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "eth-upgrade-${args.network}-state-lens-ics23-smt-client";
              value = eth-upgrade (
                {
                  dry = false;
                  protocol = "StateLensIcs23SmtClient";
                }
                // args
              );
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "eth-dryupgrade-${args.network}-ibc";
              value = eth-upgrade (
                {
                  dry = true;
                  protocol = "IBCHandler";
                }
                // args
              );
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "eth-upgrade-${args.network}-ucs00";
              value = eth-upgrade ({ protocol = "UCS00"; } // args);
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "eth-upgrade-${args.network}-cometbls-client";
              value = eth-upgrade ({ protocol = "CometblsClient"; } // args);
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "eth-get-deployed-heights-${args.network}";
              value = get-deployed-heights args;
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "eth-upgrade-${args.network}-ibc";
              value = eth-upgrade ({ protocol = "IBCHandler"; } // args);
            }) networks
          );
      };
    };
}
