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
        rev = "v0.0.253";
        hash = "sha256-P8joH3RZvA2GijTVlRE6CmSSP730Q3zY8k9jiWflyDk=";
      };
      forge-std = pkgs.fetchFromGitHub {
        owner = "foundry-rs";
        repo = "forge-std";
        rev = "v1.9.3";
        hash = "sha256-v9aFV4TQqbYPNBSRt4QLZMD85fIXTtBQ8rGYPRw2qmE=";
        fetchSubmodules = true;
      };
      openzeppelin = pkgs.fetchFromGitHub {
        owner = "OpenZeppelin";
        repo = "openzeppelin-contracts";
        rev = "v5.0.2";
        hash = "sha256-Ln721yNPzbtn36/meSmaszF6iCsJUP7iG35Je5x8x1Q=";
      };
      openzeppelin-upgradeable = pkgs.fetchFromGitHub {
        owner = "OpenZeppelin";
        repo = "openzeppelin-contracts-upgradeable";
        rev = "v5.0.2";
        hash = "sha256-/TCv1EF3HPldTsXKThuc3L2DmlyodiduSMwYymR5idM=";
      };
      openzeppelin-foundry-upgrades = pkgs.fetchFromGitHub {
        owner = "OpenZeppelin";
        repo = "openzeppelin-foundry-upgrades";
        rev = "v0.2.1";
        hash = "sha256-tQ6J5X/kpsGqHfapkDkaS2apbjL+I63vgQEk1vQI/c0=";
      };
      libraries = pkgs.linkFarm "evm-libraries" [
        {
          name = "solidity-stringutils";
          path = "${solidity-stringutils}/src";
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
          path = "${openzeppelin}/contracts";
        }
        {
          name = "@openzeppelin-upgradeable";
          path = "${openzeppelin-upgradeable}/contracts";
        }
        {
          name = "@openzeppelin-foundry-upgradeable";
          path = "${openzeppelin-foundry-upgrades}/src";
        }
      ];
      evmLibs = pkgs.stdenv.mkDerivation {
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
        fixupPhase = ''
          substituteInPlace $out/@openzeppelin-upgradeable/proxy/utils/UUPSUpgradeable.sol \
            --replace 'openzeppelin/contracts' 'openzeppelin'

          substituteInPlace $out/@openzeppelin-foundry-upgradeable/Upgrades.sol \
            --replace 'openzeppelin/contracts' 'openzeppelin'
          substituteInPlace $out/@openzeppelin-foundry-upgradeable/Upgrades.sol \
            --replace 'solidity-stringutils/src' 'solidity-stringutils'

          substituteInPlace $out/@openzeppelin-foundry-upgradeable/internal/Utils.sol \
            --replace 'solidity-stringutils/src' 'solidity-stringutils'

          substituteInPlace $out/@openzeppelin-foundry-upgradeable/internal/DefenderDeploy.sol \
            --replace 'openzeppelin/contracts' 'openzeppelin'
          substituteInPlace $out/@openzeppelin-foundry-upgradeable/internal/DefenderDeploy.sol \
            --replace 'solidity-stringutils/src' 'solidity-stringutils'
        '';
      };
      evmSources = pkgs.stdenv.mkDerivation {
        name = "evm-union-src";
        phases = [
          "installPhase"
          "fixupPhase"
        ];
        src = evmLibs;
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
      };
      # Foundry FS permissions must be explicitly set in the config file
      foundryConfig = pkgs.writeTextDir "/foundry.toml" ''
        [profile.default.optimizer_details]
        cse = true
        constantOptimizer = true
        yul = true

        [profile.default]
        fs_permissions = [{ access = "read", path = "./"}]
        libs = ["libs"]
        gas_reports = ["*"]
        via_ir = true
        ast = true
        optimizer = true
        optimizer_runs = 1_000

        [profile.script]
        src = "scripts"
        bytecode_hash = "none"
        cbor_metadata = false
        sparse_mode = false

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
          rpc-url = "https://rpc-sepolia.rockx.com";
          private-key = ''"$1"'';
          extra-args = ''--verify --verifier etherscan --etherscan-api-key "$2"'';
        }
        {
          network = "holesky";
          rpc-url = "https://holesky.drpc.org";
          private-key = ''"$1"'';
          extra-args = ''--verify --verifier etherscan --etherscan-api-key "$2"'';
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

      eth-deploy-multicall =
        {
          rpc-url,
          kind,
          extra-args ? "",
          ...
        }:
        mkCi false (
          pkgs.writeShellApplicationWithArgs {
            name = "eth-deploy-multicall";
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
            ];
            text = ''
              ${ensureAtRepositoryRoot}
              OUT="$(mktemp -d)"
              pushd "$OUT"
              cp --no-preserve=mode -r ${self'.packages.evm-contracts}/* .
              cp --no-preserve=mode -r ${evmSources}/* .

              DEPLOYER="$argc_deployer_pk" \
              PRIVATE_KEY="$argc_private_key" \
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
              ++ pkgs.lib.optional (!dry) {
                arg = "private_key";
                required = true;
                help = "The contract owner private key.";
              }
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
              PRIVATE_KEY="${pkgs.lib.optionalString (!dry) "$argc_private_key"}" \
              FOUNDRY_PROFILE="script" forge script scripts/Deploy.s.sol:${pkgs.lib.optionalString dry "Dry"}Upgrade${protocol} -vvvvv \
                --rpc-url ${rpc-url} \
                --broadcast

              rm -rf "$OUT"
              popd
            '';
          }
        );
    in
    {
      packages =
        {
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

          # Stack too deep :)
          #
          solidity-coverage =
            pkgs.runCommand "solidity-coverage"
              {
                buildInputs = [
                  self'.packages.forge
                  pkgs.lcov
                ];
              }
              ''
                  cp --no-preserve=mode -r ${evmSources}/* .
                  FOUNDRY_PROFILE="test" forge coverage --ir-minimum --report lcov
                  lcov --remove ./lcov.info -o ./lcov.info.pruned \
                    'contracts/Multicall.sol' \
                    'contracts/clients/Verifier.sol' \
                    'contracts/apps/ucs/00-pingpong/*' \
                    'contracts/lib/*' \
                    'contracts/core/OwnableIBCHandler.sol' \
                    'contracts/core/24-host/IBCCommitment.sol' \
                    'contracts/core/25-handler/IBCHandler.sol' \
                    'contracts/clients/ICS23MembershipVerifier.sol' \
                    'tests/*'
                  genhtml lcov.info.pruned -o $out --branch-coverage
                mv lcov.info.pruned $out/lcov.info
              '';
          show-solidity-coverage = pkgs.writeShellApplication {
            name = "show-solidity-coverage";
            runtimeInputs = [ ];
            text = ''
              xdg-open ${self'.packages.solidity-coverage}/index.html
            '';
          };

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
                    ${contracts}/out/IBCClient.sol/IBCClient.json \
                    ${contracts}/out/IBCPacket.sol/IBCPacket.json \
                    ${contracts}/out/IBCConnection.sol/IBCConnection.json \
                    ${contracts}/out/OwnableIBCHandler.sol/OwnableIBCHandler.json \
                    ${contracts}/out/IBCChannel.sol/IBCChannelHandshake.json > ibc-handler.json

                  jq --compact-output --slurp 'map(.abi) | add' \
                    ${contracts}/out/Relay.sol/IRelay.json \
                    ${contracts}/out/Relay.sol/UCS01Relay.json \
                    ${contracts}/out/Relay.sol/RelayLib.json \
                    ${contracts}/out/Relay.sol/RelayPacketLib.json > ucs-01.json

                  jq --compact-output --slurp 'map(.abi) | add' \
                    ${contracts}/out/NFT.sol/NFTLib.json \
                    ${contracts}/out/NFT.sol/NFTPacketLib.json \
                    ${contracts}/out/NFT.sol/UCS02NFT.json > ucs-02.json
                ''
            );

          solidity-build-tests = pkgs.writeShellApplication {
            name = "run-solidity-build-tests";
            runtimeInputs = [ self'.packages.forge ];
            text = ''
              ${ensureAtRepositoryRoot}
              FOUNDRY_LIBS=["${evmLibs}"] FOUNDRY_PROFILE="test" FOUNDRY_TEST="evm/tests/src" forge test -vvvv --match-path evm/tests/src/02-client/CosmosInCosmosClient.t.sol --gas-report "$@"
            '';
          };

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

                DEPLOYER="$1" SENDER="$2" FOUNDRY_PROFILE="script" forge script scripts/Deploy.s.sol:GetDeployed -vvv

                rm -rf "$OUT"
                popd
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
        }
        // builtins.listToAttrs (
          builtins.map (args: {
            name = "eth-deploy-${args.network}-full";
            value = eth-deploy-full args;
          }) networks
        )
        // builtins.listToAttrs (
          builtins.map (args: {
            name = "eth-deploy-${args.network}-multicall";
            value = eth-deploy-multicall ({ kind = "Multicall"; } // args);
          }) networks
        )
        // builtins.listToAttrs (
          builtins.map (args: {
            name = "eth-dryupgrade-${args.network}-ucs01";
            value = eth-upgrade (
              {
                dry = true;
                protocol = "UCS01";
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
            name = "eth-upgrade-${args.network}-ucs01";
            value = eth-upgrade ({ protocol = "UCS01"; } // args);
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
            name = "eth-upgrade-${args.network}-ibc";
            value = eth-upgrade ({ protocol = "IBCHandler"; } // args);
          }) networks
        );
    };
}
