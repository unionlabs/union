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
      gitRevToUse = gitRev;
      # use this to override the git rev. useful if verifying a contract off of a commit and the worktree is dirty for unrelated reasons (changing an rpc, adding a new explorer to verify on, etc)
      # gitRevToUse = "";

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
              --replace-fail 'dirty' '${gitRevToUse}'
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

        [etherscan]
        local-devnet = { key = "''${VERIFICATION_KEY}", chain = "32382", url = "http://localhost/api" }
        corn-testnet = { key = "''${VERIFICATION_KEY}", chain = "21000001", url = "https://api.tenderly.co/api/v1/account/unionlabs/project/union/etherscan/verify/network/21000001/public" }
        bob-mainnet = { key = "''${VERIFICATION_KEY}", chain = "60808", url = "https://api.tenderly.co/api/v1/account/unionlabs/project/union/etherscan/verify/network/60808/public" }
        bob-testnet = { key = "''${VERIFICATION_KEY}", chain = "808813", url = "https://api.tenderly.co/api/v1/account/unionlabs/project/union/etherscan/verify/network/808813/public" }
        bepolia = { key = "''${VERIFICATION_KEY}", chain = "80069", url = "https://api.routescan.io/v2/network/testnet/evm/80069/etherscan" }
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

      # network           : plaintext name of network
      # rpc-url           : rpc url for this network, should support full eth_getLogs (for fetching the
      #                     deployment heights)
      # private-key       : bash expression that evaluates to the private key to use for deployments
      #
      # verify            : whether this chain supports verification. defaults to true, if true then the
      #                     following args are also read:
      # verifier          : forge --verifier to use
      # verification-key  : bash expression that evaluates to the verification key, this will be available
      #                     in the $VERIFICATION_KEY env var
      networks = [
        # devnets
        {
          network = "devnet";
          rpc-url = "http://localhost:8545";
          private-key = "0x${builtins.readFile ./../networks/genesis/devnet-eth/dev-key0.prv}";
          weth = "0x0000000000000000000000000000000000000000";

          verify = pkgs.lib.optionalString pkgs.stdenv.isx86_64;
          verifier = "blockscout";
          verification-key = "";
        }
        {
          # for use with the local berachain devnet from berachain/beacon-kit
          network = "berachain-devnet";
          rpc-url = "http://localhost:8545";
          private-key = "0xfffdbb37105441e14b0ee6330d855d8504ff39e705c3afa8f859ac9865f99306";
          weth = "0x0000000000000000000000000000000000000000";

          verify = false;
        }
        {
          # for use with the local arbitrum devnet from offchainlabs/nitro-testnode
          network = "arbitrum-devnet";
          rpc-url = "http://localhost:8547";
          private-key = "0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659";
          weth = "0x0000000000000000000000000000000000000000";

          verify = false;
        }

        # testnets
        {
          network = "sepolia";
          rpc-url = "https://0xrpc.io/sep";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0x7b79995e5f793a07bc00c21412e50ecae098e7f9";

          verifier = "etherscan";
          verification-key = ''"$1"'';
        }
        {
          network = "holesky";
          rpc-url = "https://holesky.gateway.tenderly.co";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0x94373a4919b3240d86ea41593d5eba789fef3848";

          verifier = "etherscan";
          verification-key = ''"$1"'';
        }
        {
          network = "corn-testnet";
          rpc-url = "https://testnet.corn-rpc.com";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          # TODO: find out
          weth = "0x0000000000000000000000000000000000000000";

          verifier = "etherscan";
          verification-key = ''"$(op item get tenderly --vault union-testnet-10 --field contract-verification-api-key --reveal)"'';
        }
        {
          network = "bob-testnet";
          rpc-url = "https://bob-sepolia.rpc.gobob.xyz";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0x4200000000000000000000000000000000000006";

          verifier = "etherscan";
          verification-key = ''"$(op item get tenderly --vault union-testnet-10 --field contract-verification-api-key --reveal)"'';
        }
        {
          network = "bepolia";
          rpc-url = "https://bepolia.rpc.berachain.com/";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0x6969696969696969696969696969696969696969";

          verifier = "etherscan";
          verification-key = ''verifyContract'';
        }
        {
          network = "0g-testnet";
          rpc-url = "https://evmrpc-testnet.0g.ai";
          private-key = ''"$1"'';
          # TODO: find out
          weth = "0x0000000000000000000000000000000000000000";

          verify = false;
        }

        # mainnets
        {
          network = "bob";
          rpc-url = "https://rpc.gobob.xyz";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0x4200000000000000000000000000000000000006";

          verifier = "etherscan";
          verification-key = ''"$(op item get tenderly --vault union-testnet-10 --field contract-verification-api-key --reveal)"'';
        }

        # NOTE: These haven't been tested since testnet 8 (or earlier), and as such are unlikely to work properly
        {
          network = "scroll-testnet";
          rpc-url = "https://sepolia-rpc.scroll.io";
          private-key = ''"$1"'';
          # TODO: find out
          weth = "0x0000000000000000000000000000000000000000";

          verifier = ''--verify --verifier etherscan --verifier-url https://api-sepolia.scrollscan.com/api --etherscan-api-key "$2"'';
        }
        {
          network = "arbitrum-testnet";
          rpc-url = "https://sepolia-rollup.arbitrum.io/rpc";
          private-key = ''"$1"'';
          weth = "0x980b62da83eff3d4576c647993b0c1d7faf17c73";
        }
        {
          network = "berachain-testnet";
          rpc-url = "https://fabled-serene-mountain.bera-bartio.quiknode.pro/6ab3f499dcce3d52591ce97a5f07a13fae75deb1/";
          private-key = ''"$1"'';
          # TODO: find out
          weth = "0x0000000000000000000000000000000000000000";
        }
      ];

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

            DEPLOYMENTS_FILE="deployments/deployments.json"
            export DEPLOYMENTS_FILE

            CHAIN_ID="$(cast chain-id)"
            export CHAIN_ID

            echo "chain id: $CHAIN_ID"

            jq \
              '. |= map(if .chain_id == $chain_id then .deployments.core.height = ($height | tonumber) | .deployments.core.commit = $commit else . end)' \
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
              --arg commit "$(
                cast call "$(
                  jq -r \
                    '.[] | select(.chain_id == $chain_id) | .deployments.core.address' \
                    "$DEPLOYMENTS_FILE" \
                    --arg chain_id "$CHAIN_ID"
                )" "gitRev()(string)" \
                | jq -r || echo unknown
              )" \
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
                    '. |= map(if .chain_id == $chain_id then .deployments[$key][$subkey].height = ($height | tonumber) | .deployments[$key][$subkey].commit = $commit else . end)' \
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
                    --arg commit "$(
                      cast call "$(
                        jq -r \
                          '.[] | select(.chain_id == $chain_id) | .deployments[$key][$subkey].address' \
                          "$DEPLOYMENTS_FILE" \
                          --arg chain_id "$CHAIN_ID" \
                          --arg subkey "$subkey" \
                          --arg key "$key"
                      )" "gitRev()(string)" \
                      | jq -r || echo unknown
                    )" \
                  | sponge "$DEPLOYMENTS_FILE"
                done
            done
          '';
        };

      deploy =
        {
          rpc-url,
          private-key,
          weth,

          verify ? true,
          verifier ? if verify then throw "verifier must be set in order to verify" else "",
          verification-key ? if verify then throw "verification-key must be set in order to verify" else "",
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

              WETH_ADDRESS=${weth} \
              VERIFICATION_KEY=${verification-key} \
              PRIVATE_KEY=${private-key} \
              DEPLOYER="$3" \
              FOUNDRY_PROFILE="script" \
                forge script scripts/Deploy.s.sol:DeployIBC \
                -vvvv \
                --rpc-url ${rpc-url} \
                --broadcast ${pkgs.lib.optionalString verify "--verify --verifier ${verifier}"}

              popd
              rm -rf "$OUT"
            '';
          }
        );

      deploy-full =
        {
          rpc-url,
          private-key,
          weth,

          verify ? true,
          verifier ? if verify then throw "verifier must be set in order to verify" else "",
          verification-key ? if verify then throw "verification-key must be set in order to verify" else "",
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

              WETH_ADDRESS=${weth} \
              VERIFICATION_KEY=${verification-key} \
              PRIVATE_KEY=${private-key} \
              FOUNDRY_LIBS='["libs"]' \
              FOUNDRY_PROFILE="script" \
                forge script scripts/Deploy.s.sol:DeployDeployerAndIBC \
                -vvvv \
                --rpc-url ${rpc-url} \
                --broadcast ${pkgs.lib.optionalString verify "--verify --verifier ${verifier}"}

              popd
              rm -rf "$OUT"
            '';
          }
        );

      verify =
        {
          rpc-url,
          private-key,
          weth,

          verify ? true,
          verifier ? if verify then throw "verifier must be set in order to verify" else "",
          verification-key ? if verify then throw "verification-key must be set in order to verify" else "",
          ...
        }:
        mkCi false (
          pkgs.writeShellApplication {
            name = "eth-verify";
            runtimeInputs = [ wrappedForgeOnline ];
            text = ''
              ${ensureAtRepositoryRoot}
              WETH_ADDRESS=${weth} nix run .#evm-contracts-addresses -- "$1" "$2" ${rpc-url}

              PROJECT_ROOT=$(pwd)
              OUT="$(mktemp -d)"
              pushd "$OUT"
              cp --no-preserve=mode -r ${self'.packages.evm-contracts}/* .
              cp --no-preserve=mode -r ${evmSources}/* .

              jq -r 'to_entries | map([.key, .value.args, .value.contract]) | .[] | @tsv' "$PROJECT_ROOT"/contracts.json | \
                while IFS=$'\t' read -r address args contract; do
                  if [ "$address" != "0x0000000000000000000000000000000000000000" ]
                  then
                    WETH_ADDRESS=${weth} \
                    VERIFICATION_KEY=${verification-key} \
                    PRIVATE_KEY=${private-key} \
                    FOUNDRY_LIBS='["libs"]' \
                    FOUNDRY_PROFILE="script" \
                      forge verify-contract \
                        --force \
                        --watch "$address" "$contract" \
                        --constructor-args "$args" \
                        --rpc-url ${rpc-url} ${pkgs.lib.optionalString verify "--verifier ${verifier}"} || true
                  fi
                done

              popd
              rm -rf "$OUT"
            '';
          }
        );

      deploy-single =
        {
          rpc-url,
          kind,
          weth,

          verify ? true,
          verifier ? if verify then throw "verifier must be set in order to verify" else "",
          verification-key ? if verify then throw "verification-key must be set in order to verify" else "",
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

              WETH_ADDRESS=${weth} \
              VERIFICATION_KEY=${verification-key} \
              DEPLOYER="$argc_deployer_pk" \
              SENDER="$argc_sender_pk" \
              PRIVATE_KEY="$argc_private_key" \
              FOUNDRY_LIBS='["libs"]' \
              FOUNDRY_PROFILE="script" \
                forge script scripts/Deploy.s.sol:Deploy${kind} \
                -vvvv \
                --rpc-url "${rpc-url}" \
                --broadcast ${pkgs.lib.optionalString verify "--verify --verifier ${verifier}"}

              popd
              rm -rf "$OUT"
            '';
          }
        );

      # TODO: Read the deployments.json to get the deployer and sender (can't upgrade without a deployment anyways)
      upgrade =
        {
          dry ? false,
          rpc-url,
          protocol,
          private-key,
          weth,

          verify ? true,
          verifier ? if verify then throw "verifier must be set in order to verify" else "",
          verification-key ? if verify then throw "verification-key must be set in order to verify" else "",
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

              WETH_ADDRESS=${weth} \
              VERIFICATION_KEY=${verification-key} \
              DEPLOYER="$argc_deployer_pk" \
              SENDER="$argc_sender_pk" \
              OWNER="${pkgs.lib.optionalString dry "$argc_owner_pk"}" \
              PRIVATE_KEY=${private-key} \
              FOUNDRY_LIBS='["libs"]' \
              FOUNDRY_PROFILE="script" \
                forge script scripts/Deploy.s.sol:${pkgs.lib.optionalString dry "Dry"}Upgrade${protocol} -vvvvv \
                  --rpc-url ${rpc-url} \
                  --broadcast ${pkgs.lib.optionalString verify "--verify --verifier ${verifier}"}

              rm -rf "$OUT"
              popd
            '';
          }
        );
    in
    {
      packages = {
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

        # USAGE: evm-contracts-addresses deployer sender rpc-url
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
              name = "verify-${args.network}";
              value = verify args;
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "deploy-full-${args.network}";
              value = deploy-full args;
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "deploy-${args.network}";
              value = deploy args;
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "deploy-${args.network}-state-lens-ics23-mpt-client";
              value = deploy-single ({ kind = "StateLensIcs23MptClient"; } // args);
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "deploy-${args.network}-state-lens-ics23-ics23-client";
              value = deploy-single ({ kind = "StateLensIcs23Ics23Client"; } // args);
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "deploy-${args.network}-state-lens-ics23-smt-client";
              value = deploy-single ({ kind = "StateLensIcs23SmtClient"; } // args);
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "deploy-${args.network}-ucs03";
              value = deploy-single ({ kind = "UCS03"; } // args);
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "deploy-${args.network}-multicall";
              value = deploy-single ({ kind = "Multicall"; } // args);
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "dryupgrade-${args.network}-ucs03";
              value = upgrade (
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
              name = "dryupgrade-${args.network}-cometbls-client";
              value = upgrade (
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
              name = "upgrade-${args.network}-ucs03";
              value = upgrade (
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
              name = "upgrade-${args.network}-state-lens-ics23-mpt-client";
              value = upgrade (
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
              name = "upgrade-${args.network}-state-lens-ics23-ics23-client";
              value = upgrade (
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
              name = "upgrade-${args.network}-state-lens-ics23-smt-client";
              value = upgrade (
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
              name = "dryupgrade-${args.network}-ibc";
              value = upgrade (
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
              name = "upgrade-${args.network}-ucs00";
              value = upgrade ({ protocol = "UCS00"; } // args);
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "upgrade-${args.network}-cometbls-client";
              value = upgrade ({ protocol = "CometblsClient"; } // args);
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "get-deployed-heights-${args.network}";
              value = get-deployed-heights args;
            }) networks
          )
          // builtins.listToAttrs (
            builtins.map (args: {
              name = "upgrade-${args.network}-ibc";
              value = upgrade ({ protocol = "IBCHandler"; } // args);
            }) networks
          );
      };
    };
}
