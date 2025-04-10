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
      dbg,
      ...
    }:
    let
      gitRevToUse = gitRev;
      # use this to override the git rev. useful if verifying a contract off of a commit and the worktree is dirty for unrelated reasons (for example, changing an rpc)
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
              exclude = [
                "evm.nix"
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
        [profile.default]
        fs_permissions = [{ access = "read", path = "./" }, { access = "write", path = "contracts.json" }]
        libs = ["libs"]
        gas_reports = ["*"]
        via_ir = true
        ast = true
        optimizer = true
        optimizer_runs = 10_000

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

      mkTenderlyVerifierUrl =
        chain-id:
        "https://api.tenderly.co/api/v1/account/unionlabs/project/union/etherscan/verify/network/${chain-id}/public";

      # name              : plaintext name of network
      # chain-id          : chain id of the network
      # rpc-url           : rpc url for this network, should support full eth_getLogs (for fetching the
      #                     deployment heights)
      # private-key       : bash expression that evaluates to the private key to use for deployments
      # weth              : address of the WETH equivalent on this chain, to use for ucs03-zkgm
      #
      # verify            : whether this chain supports verification. defaults to true, if true then the
      #                     following args are also read:
      # verifier          : forge --verifier to use
      # verification-key  : bash expression that evaluates to the verification key, this will be available
      #                     in the $VERIFICATION_KEY env var
      # verifier-url      : contract verification endpoint for this chain
      networks = [
        # devnets
        {
          chain-id = "32382";

          name = "devnet";
          rpc-url = "http://localhost:8545";
          private-key = "0x${builtins.readFile ./../networks/genesis/devnet-eth/dev-key0.prv}";
          weth = "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF";

          verify = pkgs.stdenv.isx86_64;
          verifier = "blockscout";
          verification-key = ''""'';
          verifier-url = "http://localhost/api";
        }
        # {
        #   # for use with the local berachain devnet from berachain/beacon-kit
        #   name = "berachain-devnet";
        #   rpc-url = "http://localhost:8545";
        #   private-key = "0xfffdbb37105441e14b0ee6330d855d8504ff39e705c3afa8f859ac9865f99306";

        #   verify = false;
        # }
        # {
        #   # for use with the local arbitrum devnet from offchainlabs/nitro-testnode
        #   name = "arbitrum-devnet";
        #   rpc-url = "http://localhost:8547";
        #   private-key = "0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659";

        #   verify = false;
        # }

        # testnets
        {
          chain-id = "11155111";

          name = "sepolia";
          rpc-url = "https://0xrpc.io/sep";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0x7b79995e5f793a07bc00c21412e50ecae098e7f9";

          verifier = "etherscan";
          verification-key = ''"''${ETHERSCAN_KEY:-etherscan api key must be set in the ETHERSCAN_KEY env var}"'';
          verifier-url = "https://api.etherscan.io/api";
        }
        {
          chain-id = "17000";

          name = "holesky";
          rpc-url = "https://holesky.gateway.tenderly.co";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0x94373a4919b3240d86ea41593d5eba789fef3848";

          verifier = "etherscan";
          verification-key = ''"''${ETHERSCAN_KEY:-etherscan api key must be set in the ETHERSCAN_KEY env var}"'';
          verifier-url = "https://api.etherscan.io/api";
        }
        rec {
          chain-id = "21000001";

          name = "corn-testnet";
          rpc-url = "https://testnet.corn-rpc.com";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0xda5dDd7270381A7C2717aD10D1c0ecB19e3CDFb2";

          verifier = "etherscan";
          verification-key = ''"$(op item get tenderly --vault union-testnet-10 --field contract-verification-api-key --reveal)"'';
          verifier-url = mkTenderlyVerifierUrl chain-id;
        }
        rec {
          chain-id = "808813";

          name = "bob-testnet";
          rpc-url = "https://bob-sepolia.rpc.gobob.xyz";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0x4200000000000000000000000000000000000006";

          verifier = "etherscan";
          verification-key = ''"$(op item get tenderly --vault union-testnet-10 --field contract-verification-api-key --reveal)"'';
          verifier-url = mkTenderlyVerifierUrl chain-id;
        }
        rec {
          chain-id = "80069";

          name = "bepolia";
          rpc-url = "https://bepolia.rpc.berachain.com/";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0x6969696969696969696969696969696969696969";

          verifier = "etherscan";
          verification-key = ''verifyContract'';
          verifier-url = "https://api.routescan.io/v2/network/testnet/evm/${chain-id}/etherscan";
        }
        # {
        #   network = "0g-testnet";
        #   rpc-url = "https://evmrpc-testnet.0g.ai";
        #   private-key = ''"$1"'';

        #   verify = false;
        # }

        # mainnets
        rec {
          chain-id = "60808";

          name = "bob";
          rpc-url = "https://rpc.gobob.xyz";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0x4200000000000000000000000000000000000006";

          verifier = "etherscan";
          verification-key = ''"$(op item get tenderly --vault union-testnet-10 --field contract-verification-api-key --reveal)"'';
          verifier-url = mkTenderlyVerifierUrl chain-id;
        }

        # NOTE: These haven't been tested since testnet 8 (or earlier), and as such are unlikely to work properly
        # {
        #   network = "scroll-testnet";
        #   rpc-url = "https://sepolia-rpc.scroll.io";
        #   private-key = ''"$1"'';

        #   verifier = ''--verify --verifier etherscan --verifier-url https://api-sepolia.scrollscan.com/api --etherscan-api-key "$2"'';
        # }
        # {
        #   network = "arbitrum-testnet";
        #   rpc-url = "https://sepolia-rollup.arbitrum.io/rpc";
        #   private-key = ''"$1"'';
        #   weth = "0x980b62da83eff3d4576c647993b0c1d7faf17c73";
        # }
        # {
        #   network = "berachain-testnet";
        #   rpc-url = "https://fabled-serene-mountain.bera-bartio.quiknode.pro/6ab3f499dcce3d52591ce97a5f07a13fae75deb1/";
        #   private-key = ''"$1"'';
        # }
      ];

      # use in a script that can do contract verification. this allows for overwriting the verification args via the FOUNDRY_ETHERSCAN env var when calling said script via nix run.
      #
      # this also allows for overwriting the verifier via the $VERIFIER env var when calling said script via nix run.
      #
      # the verification key is expected to be available at $VERIFICATION_KEY
      #
      # the args to pass to the forge invocation are available in an array at "${VERIFICATION_ARGS[@]}"
      #
      # example:
      #
      # FOUNDRY_ETHERSCAN='{ chain = { key = "verifyContract", chain = "21000001", url = "https://api.routescan.io/v2/network/testnet/evm/21000001/etherscan" } }' nix run .#evm-scripts.verify-corn-testnet -L -- 0xa76897C61d710C07De4D541C77c209578d64CEB9 0x95Fb5cb304508d74d855514D7bC9bDA75c304cE2
      setupFoundryVerifcationVars =
        verify:
        {
          chain-id,
          verifier,
          verifier-url,
          with-verify-flag ? true,
        }:
        let
          expr =
            if verify then
              ''{ chain = { key = \"\''${VERIFICATION_KEY}\", chain = \"${chain-id}\", url = \"${verifier-url}\" } }''
            else
              "{}";
        in
        ''
          # shellcheck disable=SC2016
          DEFAULT_FOUNDRY_ETHERSCAN="${expr}"
          FOUNDRY_ETHERSCAN="''${FOUNDRY_ETHERSCAN:-$DEFAULT_FOUNDRY_ETHERSCAN}"

          echo "$FOUNDRY_ETHERSCAN"

          VERIFICATION_ARGS=()
          # shellcheck disable=2050
          # idk how else to compare against a bool from nix -> bash
          if [ ${if verify then "1" else "0"} -eq 1 ] || [ -z "''${VERIFIER:-}" ]; then
            # either default verifier, or specified verifier
            if [ ${if with-verify-flag then "1" else "0"} -eq 1 ]; then
              VERIFICATION_ARGS+=("--verify")
            fi
            VERIFICATION_ARGS+=("--verifier")
            VERIFICATION_ARGS+=("''${VERIFIER:-${verifier}}")
          fi
        '';

      update-deployments-json =
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
          name,

          chain-id,
          rpc-url,
          private-key,
          weth,

          verify ? true,
          verifier ? if verify then throw "verifier must be set in order to verify" else "",
          verification-key ? if verify then throw "verification-key must be set in order to verify" else "",
          verifier-url ? if verify then throw "verifier-url must be set in order to verify" else "",
          ...
        }:
        mkCi false (
          pkgs.writeShellApplication {
            name = "eth-deploy-${name}";
            runtimeInputs = [ self'.packages.forge ];
            text = ''
              ${ensureAtRepositoryRoot}

              ${setupFoundryVerifcationVars verify {
                inherit chain-id verifier verifier-url;
              }}

              OUT="$(mktemp -d)"
              pushd "$OUT"
              cp --no-preserve=mode -r ${self'.packages.evm-contracts}/* .
              cp --no-preserve=mode -r ${evmSources}/* .

              FOUNDRY_ETHERSCAN="$FOUNDRY_ETHERSCAN" \
              VERIFICATION_KEY=${verification-key} \
              WETH_ADDRESS=${weth} \
              PRIVATE_KEY=${private-key} \
              DEPLOYER="''${1:?deployer must be set to deploy with this script (first arg to this script)}" \
              FOUNDRY_PROFILE="script" \
                forge script scripts/Deploy.s.sol:DeployIBC \
                -vvvv \
                --rpc-url ${rpc-url} \
                --broadcast "''${VERIFICATION_ARGS[@]}"

              popd
              rm -rf "$OUT"
            '';
          }
        );

      deploy-deployer-and-ibc =
        {
          chain-id,
          name,
          rpc-url,
          private-key,
          weth,

          verify ? true,
          verifier ? if verify then throw "verifier must be set in order to verify" else "",
          verification-key ? if verify then throw "verification-key must be set in order to verify" else "",
          verifier-url ? if verify then throw "verifier-url must be set in order to verify" else "",
          ...
        }:
        mkCi false (
          pkgs.writeShellApplication {
            name = "eth-deploy-deployer-and-ibc-${name}";
            runtimeInputs = [ self'.packages.forge ];
            text = ''
              ${ensureAtRepositoryRoot}

              ${setupFoundryVerifcationVars verify {
                inherit chain-id verifier verifier-url;
              }}

              OUT="$(mktemp -d)"
              pushd "$OUT"
              cp --no-preserve=mode -r ${self'.packages.evm-contracts}/* .
              cp --no-preserve=mode -r ${evmSources}/* .

              FOUNDRY_ETHERSCAN="$FOUNDRY_ETHERSCAN" \
              VERIFICATION_KEY=${verification-key} \
              WETH_ADDRESS=${weth} \
              PRIVATE_KEY=${private-key} \
              FOUNDRY_LIBS='["libs"]' \
              FOUNDRY_PROFILE="script" \
                forge script scripts/Deploy.s.sol:DeployDeployerAndIBC \
                -vvvv \
                --rpc-url ${rpc-url} \
                --broadcast "''${VERIFICATION_ARGS[@]}"

              popd
              rm -rf "$OUT"
            '';
          }
        );

      verify-all-contracts =
        {
          chain-id,
          rpc-url,
          private-key,
          weth,

          verify ? true,
          verifier ? if verify then throw "verifier must be set in order to verify" else "",
          verification-key ? if verify then throw "verification-key must be set in order to verify" else "",
          verifier-url ? if verify then throw "verifier-url must be set in order to verify" else "",
          ...
        }:
        mkCi false (
          pkgs.writeShellApplication {
            name = "eth-verify";
            runtimeInputs = [ wrappedForgeOnline ];
            text = ''
              ${ensureAtRepositoryRoot}

              ${setupFoundryVerifcationVars verify {
                inherit chain-id verifier verifier-url;
                with-verify-flag = false;
              }}

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
                    # shellcheck disable=SC2005
                    FOUNDRY_ETHERSCAN="$FOUNDRY_ETHERSCAN" \
                    VERIFICATION_KEY=${verification-key} \
                    WETH_ADDRESS=${weth} \
                    PRIVATE_KEY=${private-key} \
                    FOUNDRY_LIBS='["libs"]' \
                    FOUNDRY_PROFILE="script" \
                      forge verify-contract \
                        --force \
                        --watch "$address" "$contract" \
                        --constructor-args "$args" \
                        --rpc-url ${rpc-url} "''${VERIFICATION_ARGS[@]}" || true
                  fi
                done

              popd
              rm -rf "$OUT"
            '';
          }
        );

      deploy-single =
        {
          kind,

          chain-id,
          rpc-url,
          weth,

          verify ? true,
          verifier ? if verify then throw "verifier must be set in order to verify" else "",
          verification-key ? if verify then throw "verification-key must be set in order to verify" else "",
          verifier-url ? if verify then throw "verifier-url must be set in order to verify" else "",
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

              ${setupFoundryVerifcationVars verify {
                inherit chain-id verifier verifier-url;
              }}

              OUT="$(mktemp -d)"
              pushd "$OUT"
              cp --no-preserve=mode -r ${self'.packages.evm-contracts}/* .
              cp --no-preserve=mode -r ${evmSources}/* .

              FOUNDRY_ETHERSCAN="$FOUNDRY_ETHERSCAN" \
              VERIFICATION_KEY=${verification-key} \
              WETH_ADDRESS=${weth} \
              DEPLOYER="$argc_deployer_pk" \
              SENDER="$argc_sender_pk" \
              PRIVATE_KEY="$argc_private_key" \
              FOUNDRY_LIBS='["libs"]' \
              FOUNDRY_PROFILE="script" \
                forge script scripts/Deploy.s.sol:Deploy${kind} \
                -vvvv \
                --rpc-url "${rpc-url}" \
                --broadcast "''${VERIFICATION_ARGS[@]}"

              popd
              rm -rf "$OUT"
            '';
          }
        );

      # TODO: Read the deployments.json to get the deployer and sender (can't upgrade without a deployment anyways)
      upgrade =
        {
          dry ? false,
          protocol,

          chain-id,
          private-key,
          rpc-url,
          weth,

          verify ? true,
          verifier ? if verify then throw "verifier must be set in order to verify" else "",
          verification-key ? if verify then throw "verification-key must be set in order to verify" else "",
          verifier-url ? if verify then throw "verifier-url must be set in order to verify" else "",
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

              ${setupFoundryVerifcationVars verify {
                inherit chain-id verifier verifier-url;
                with-verify-flag = false;
              }}

              WETH_ADDRESS=${weth} \
              FOUNDRY_ETHERSCAN="$FOUNDRY_ETHERSCAN" \
              VERIFICATION_KEY=${verification-key} \
              DEPLOYER="$argc_deployer_pk" \
              SENDER="$argc_sender_pk" \
              OWNER="${pkgs.lib.optionalString dry "$argc_owner_pk"}" \
              PRIVATE_KEY=${private-key} \
              FOUNDRY_LIBS='["libs"]' \
              FOUNDRY_PROFILE="script" \
                forge script scripts/Deploy.s.sol:${pkgs.lib.optionalString dry "Dry"}Upgrade${protocol} -vvvvv \
                  --rpc-url ${rpc-url} \
                  --broadcast "''${VERIFICATION_ARGS[@]}"

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
                  ${contracts}/out/IBCHandler.sol/IBCHandler.json > core.json

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

        evm-scripts = pkgs.mkRootDrv "evm-scripts" (
          builtins.listToAttrs (
            map (chain: {
              inherit (chain) name;
              value = pkgs.mkRootDrv chain.name (
                {
                  verify-all-contracts = verify-all-contracts chain;
                  deploy = deploy chain;
                  deploy-deployer-and-ibc = deploy-deployer-and-ibc chain;
                  update-deployments-json = update-deployments-json chain;
                  # finalize-deployment = finalize-deployment chain;
                  # get-git-rev = get-git-rev chain;
                }
                // (pkgs.lib.mapAttrs'
                  (name: kind: {
                    name = "deploy-${name}";
                    value = deploy-single (chain // { inherit kind; });
                  })
                  {
                    ucs03 = "UCS03";
                    cometbls-client = "CometblsClient";
                    state-lens-ics23-mpt-client = "StateLensIcs23MptClient";
                    state-lens-ics23-ics23-client = "StateLensIcs23Ics23Client";
                    state-lens-ics23-smt-client = "StateLensIcs23SmtClient";
                    multicall = "Multicall";
                  }
                )
                // (builtins.foldl' (a: b: a // b) { } (
                  pkgs.lib.flatten (
                    pkgs.lib.mapAttrsToList
                      (
                        name: protocol:
                        (map
                          (dry: {
                            ${"upgrade-${name}" + (pkgs.lib.optionalString dry "-dry")} = upgrade (
                              chain // { inherit dry protocol; }
                            );
                          })
                          [
                            true
                            false
                          ]
                        )
                      )
                      {
                        ucs00 = "UCS00";
                        ucs03 = "UCS03";
                        cometbls-client = "CometblsClient";
                        state-lens-ics23-mpt-client = "StateLensIcs23MptClient";
                        state-lens-ics23-ics23-client = "StateLensIcs23Ics23Client";
                        state-lens-ics23-smt-client = "StateLensIcs23SmtClient";
                        core = "IBCHandler";
                      }
                  )
                ))
              );
            }) networks
          )
        );
      };
    };
}
