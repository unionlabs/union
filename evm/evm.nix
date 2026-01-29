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

      # Get the full deployments object for a chain.
      getDeployment =
        ucs04-chain-id:
        (builtins.fromJSON (builtins.readFile ../deployments/deployments.json)).${ucs04-chain-id};

      # Get a deployed contract address by name on a chain.
      getDeployedContractAddress =
        ucs04-chain-id: name:
        (pkgs.lib.lists.findSingle ({ value, ... }: value.name == name)
          (throw "no deployment found for ${name} on ${ucs04-chain-id}")
          (throw "many deployments found for ${name} on ${ucs04-chain-id}")
          (pkgs.lib.attrsToList (getDeployment ucs04-chain-id).contracts)
        ).name;

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
        rev = "73f13dd1483707ef6b4d16cb0543570b7e1715a8";
        hash = "sha256-RHKgAeb0bdKjFP50k/UNcH64MZyiITrsG2JfrhNWXu8=";
      };
      forge-std = pkgs.fetchFromGitHub {
        owner = "foundry-rs";
        repo = "forge-std";
        rev = "v1.9.6";
        hash = "sha256-4y1Hf0Te2oJxwKBOgVBEHZeKYt7hs+wTgdIO+rItj0E=";
        fetchSubmodules = true;
      };
      safe-utils = pkgs.fetchFromGitHub {
        owner = "hussein-aitlahcen";
        repo = "safe-utils";
        rev = "d1ed14624352c22b1255555ad48fea1f532fb953";
        hash = "sha256-4pqAofsN2Oo9sDT1/p76lVV1dD6xgvzv8lxVtqDaN9s=";
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
          name = "@safe-utils";
          path = "${safe-utils}/src";
        }
        {
          name = "lib/safe-smart-account";
          path = "${safe-utils}/lib/safe-smart-account/contracts";
        }
        {
          name = "lib/solidity-http";
          path = "${safe-utils}/lib/solidity-http";
        }
        {
          name = "lib/solidity-stringutils";
          path = "${safe-utils}/lib/solidity-stringutils";
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
                "bridged_tokens_v1.json"
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
        bytecode_hash = "none"
        ast = true
        optimizer = true
        optimizer_runs = 10_000
        cbor_metadata = false
        sparse_mode = false
        memory_limit = 33554432
        solc_version   = "0.8.27"

        [profile.script]
        ffi = true
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

      ba5ed = "0xba5eD44733953d79717F6269357C77718C8Ba5ed";
      eu = "0xe5Cf13C84c0fEa3236C101Bd7d743d30366E5CF1";

      # name                  : plaintext name of network
      # chain-id              : chain id of the network
      # ucs04-chain-id        : ucs04 chain id of the network
      # rpc-url               : rpc url for this network, should support full eth_getLogs (for fetching the
      #                         deployment heights)
      # private-key           : bash expression that evaluates to the private key to use for deployments
      #
      # weth                  : ucs03 - address of the WETH equivalent on this chain
      # rate-limit-enabled    : ucs03 - whether rate limiting is enabled for ucs03-zkgm
      # native-token-name     : ucs03 - name of the native token on the chain
      # native-token-symbol   : ucs03 - symbol of the native token on the chain
      # native-token-decimals : ucs03 - number of decimal places for the native token
      #
      # verify                : whether this chain supports verification. defaults to true, if true then the
      #                         following args are also read:
      # verifier              : forge --verifier to use
      # verification-key      : bash expression that evaluates to the verification key, this will be available
      #                         in the $VERIFICATION_KEY env var
      # verifier-url          : contract verification endpoint for this chain
      #
      # u                     : the address of $U on this chain
      networks = [
        # devnets
        {
          chain-id = "32382";

          name = "devnet";
          rpc-url = "http://localhost:8545";
          private-key = "0x${builtins.readFile ./../networks/genesis/devnet-eth/dev-key0.prv}";
          weth = "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF";
          rate-limit-enabled = "false";

          native-token-name = "Ether";
          native-token-symbol = "ETH";
          native-token-decimals = 18;

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
        rec {
          chain-id = "11155111";
          ucs04-chain-id = "ethereum.11155111";

          name = "sepolia";
          rpc-url = "https://eth-sepolia.g.alchemy.com/v2/Xn_VBUDyUtXUYb9O6b5ZmuBNDaSlH-BB";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0x7b79995e5f793a07bc00c21412e50ecae098e7f9";
          rate-limit-enabled = "false";

          native-token-name = "Ether";
          native-token-symbol = "ETH";
          native-token-decimals = 18;

          verifier = "etherscan";
          verification-key = ''"$(op item get tenderly --vault union-testnet-10 --field contract-verification-api-key --reveal)"'';
          verifier-url = mkTenderlyVerifierUrl chain-id;

          u = ba5ed;
          inherit eu;
        }
        rec {
          chain-id = "17000";
          ucs04-chain-id = "ethereum.17000";

          name = "holesky";
          rpc-url = "https://eth-holesky.g.alchemy.com/v2/Xn_VBUDyUtXUYb9O6b5ZmuBNDaSlH-BB";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0x94373a4919b3240d86ea41593d5eba789fef3848";
          rate-limit-enabled = "false";

          native-token-name = "Ether";
          native-token-symbol = "ETH";
          native-token-decimals = 18;

          verifier = "etherscan";
          verification-key = ''"$(op item get tenderly --vault union-testnet-10 --field contract-verification-api-key --reveal)"'';
          verifier-url = mkTenderlyVerifierUrl chain-id;

          u = ba5ed;
          inherit eu;
        }
        rec {
          chain-id = "21000001";
          ucs04-chain-id = "corn.21000001";

          name = "corn-testnet";
          rpc-url = "https://testnet.corn-rpc.com";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0xda5dDd7270381A7C2717aD10D1c0ecB19e3CDFb2";
          rate-limit-enabled = "false";
          native-token-name = "Bitcorn";
          native-token-symbol = "BTCN";
          native-token-decimals = 18;

          verifier = "etherscan";
          verification-key = ''"$(op item get tenderly --vault union-testnet-10 --field contract-verification-api-key --reveal)"'';
          verifier-url = mkTenderlyVerifierUrl chain-id;
        }
        rec {
          chain-id = "808813";
          ucs04-chain-id = "bob.808813";

          name = "bob-sepolia";
          rpc-url = "https://bob-sepolia.rpc.gobob.xyz";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0x4200000000000000000000000000000000000006";
          rate-limit-enabled = "false";

          native-token-name = "Ether";
          native-token-symbol = "ETH";
          native-token-decimals = 18;

          verifier = "etherscan";
          verification-key = ''"$(op item get tenderly --vault union-testnet-10 --field contract-verification-api-key --reveal)"'';
          verifier-url = mkTenderlyVerifierUrl chain-id;
        }
        rec {
          chain-id = "80069";
          ucs04-chain-id = "berachain.80069";

          name = "bepolia";
          rpc-url = "https://bepolia.rpc.berachain.com/";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0x6969696969696969696969696969696969696969";
          rate-limit-enabled = "false";
          native-token-name = "Bera";
          native-token-symbol = "BERA";
          native-token-decimals = 18;

          verifier = "etherscan";
          verification-key = ''"$(op item get tenderly --vault union-testnet-10 --field contract-verification-api-key --reveal)"'';
          verifier-url = mkTenderlyVerifierUrl chain-id;
        }
        rec {
          chain-id = "1328";
          ucs04-chain-id = "sei.1328";

          name = "sei-atlantic";
          rpc-url = "https://evm-rpc-testnet.sei-apis.com";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0xDc78B593dD44914C326D1ed37501EAd48c4C5628";
          rate-limit-enabled = "false";

          native-token-name = "Sei";
          native-token-symbol = "SEI";
          native-token-decimals = 18;

          verifier = "etherscan";
          verification-key = ''"$(op item get tenderly --vault union-testnet-10 --field contract-verification-api-key --reveal)"'';
          verifier-url = mkTenderlyVerifierUrl chain-id;
        }
        rec {
          chain-id = "97";
          ucs04-chain-id = "bsc.97";

          name = "bsc-testnet";
          rpc-url = "https://bsc-testnet.bnbchain.org";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0xae13d989dac2f0debff460ac112a837c89baa7cd";
          rate-limit-enabled = "false";

          native-token-name = "BNB";
          native-token-symbol = "BNB";
          native-token-decimals = 18;

          verifier = "etherscan";
          verification-key = ''"$(op item get tenderly --vault union-testnet-10 --field contract-verification-api-key --reveal)"'';
          verifier-url = mkTenderlyVerifierUrl chain-id;
        }
        rec {
          chain-id = "84532";
          ucs04-chain-id = "base.84532";

          name = "base-sepolia";
          rpc-url = "https://sepolia.base.org";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0x4200000000000000000000000000000000000006";
          rate-limit-enabled = "false";

          native-token-name = "Ether";
          native-token-symbol = "ETH";
          native-token-decimals = 18;

          verifier = "etherscan";
          verification-key = ''"$(op item get tenderly --vault union-testnet-10 --field contract-verification-api-key --reveal)"'';
          verifier-url = mkTenderlyVerifierUrl chain-id;
        }
        rec {
          chain-id = "421614";
          ucs04-chain-id = "arbitrum.421614";

          name = "arbitrum-sepolia";
          rpc-url = "https://arb-sepolia.g.alchemy.com/v2/Xn_VBUDyUtXUYb9O6b5ZmuBNDaSlH-BB";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0x980B62Da83eFf3D4576C647993b0c1D7faf17c73";
          rate-limit-enabled = "false";

          native-token-name = "Ether";
          native-token-symbol = "ETH";
          native-token-decimals = 18;

          verifier = "etherscan";
          verification-key = ''"$(op item get tenderly --vault union-testnet-10 --field contract-verification-api-key --reveal)"'';
          verifier-url = mkTenderlyVerifierUrl chain-id;
        }
        # {
        #   network = "0g-testnet";
        #   rpc-url = "https://evmrpc-testnet.0g.ai";
        #   private-key = ''"$1"'';

        #   verify = false;
        # }

        # mainnets
        rec {
          chain-id = "1";
          ucs04-chain-id = "ethereum.1";

          name = "ethereum";
          rpc-url = "https://eth-mainnet.g.alchemy.com/v2/Xn_VBUDyUtXUYb9O6b5ZmuBNDaSlH-BB";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";

          native-token-name = "Ether";
          native-token-symbol = "ETH";
          native-token-decimals = 18;

          verifier = "etherscan";
          verification-key = ''"$(op item get tenderly --vault union-testnet-10 --field contract-verification-api-key --reveal)"'';
          verifier-url = mkTenderlyVerifierUrl chain-id;

          u = ba5ed;
        }
        rec {
          chain-id = "60808";
          ucs04-chain-id = "bob.60808";

          name = "bob";
          rpc-url = "https://rpc.gobob.xyz";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0x4200000000000000000000000000000000000006";

          native-token-name = "Ether";
          native-token-symbol = "ETH";
          native-token-decimals = 18;

          verifier = "etherscan";
          verification-key = ''"$(op item get tenderly --vault union-testnet-10 --field contract-verification-api-key --reveal)"'';
          verifier-url = mkTenderlyVerifierUrl chain-id;
        }
        rec {
          chain-id = "21000000";
          ucs04-chain-id = "corn.21000000";

          name = "corn";
          rpc-url = "https://mainnet.corn-rpc.com";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0xda5dDd7270381A7C2717aD10D1c0ecB19e3CDFb2";
          native-token-name = "Bitcorn";
          native-token-symbol = "BTCN";
          native-token-decimals = 18;

          verifier = "etherscan";
          verification-key = ''"$(op item get tenderly --vault union-testnet-10 --field contract-verification-api-key --reveal)"'';
          verifier-url = mkTenderlyVerifierUrl chain-id;
        }
        rec {
          chain-id = "56";
          ucs04-chain-id = "bsc.56";

          name = "bsc";
          rpc-url = "https://bsc-rpc.publicnode.com";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';

          # Wrapped BNB
          weth = "0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c";

          native-token-name = "BNB";
          native-token-symbol = "BNB";
          native-token-decimals = 18;

          verifier = "etherscan";
          verification-key = ''"$(op item get tenderly --vault union-testnet-10 --field contract-verification-api-key --reveal)"'';
          verifier-url = mkTenderlyVerifierUrl chain-id;

          u = ba5ed;
        }
        rec {
          chain-id = "8453";
          ucs04-chain-id = "base.8453";

          name = "base";
          rpc-url = "https://base-mainnet.g.alchemy.com/v2/Xn_VBUDyUtXUYb9O6b5ZmuBNDaSlH-BB";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0x4200000000000000000000000000000000000006";
          rate-limit-enabled = "true";

          native-token-name = "Ether";
          native-token-symbol = "ETH";
          native-token-decimals = 18;

          verifier = "etherscan";
          verification-key = ''"$(op item get tenderly --vault union-testnet-10 --field contract-verification-api-key --reveal)"'';

          verifier-url = mkTenderlyVerifierUrl chain-id;

          u = ba5ed;
        }
        rec {
          chain-id = "1329";
          ucs04-chain-id = "sei.1329";

          name = "sei";
          rpc-url = "https://sei-mainnet.g.alchemy.com/v2/Xn_VBUDyUtXUYb9O6b5ZmuBNDaSlH-BB";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0xe30fedd158a2e3b13e9badaeabafc5516e95e8c7";
          rate-limit-enabled = "false";

          native-token-name = "Sei";
          native-token-symbol = "SEI";
          native-token-decimals = 18;

          verifier = "etherscan";
          verification-key = ''"$(op item get tenderly --vault union-testnet-10 --field contract-verification-api-key --reveal)"'';
          verifier-url = mkTenderlyVerifierUrl chain-id;

          u = ba5ed;
        }
        rec {
          chain-id = "42161";
          ucs04-chain-id = "arbitrum.42161";

          name = "arbitrum";
          rpc-url = "https://arb-mainnet.g.alchemy.com/v2/Xn_VBUDyUtXUYb9O6b5ZmuBNDaSlH-BB";
          private-key = ''"$(op item get deployer --vault union-testnet-10 --field evm-private-key --reveal)"'';
          weth = "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1";
          rate-limit-enabled = "true";

          native-token-name = "Ether";
          native-token-symbol = "ETH";
          native-token-decimals = 18;

          verifier = "etherscan";
          verification-key = ''"$(op item get tenderly --vault union-testnet-10 --field contract-verification-api-key --reveal)"'';
          verifier-url = mkTenderlyVerifierUrl chain-id;
        }
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
        {
          rpc-url,
          ucs04-chain-id,
          name,
          u ? null,
          eu ? null,
          ...
        }:
        pkgs.writeShellApplication {
          name = "update-deployments-json-${name}";
          runtimeInputs = [
            self'.packages.update-deployments
          ];
          text = ''
            ${ensureAtRepositoryRoot}

            RUST_LOG=info update-deployments \
              "deployments/deployments.json" \
              ${ucs04-chain-id} \
              --rpc-url "''${RPC_URL:-${rpc-url}}" \
              --lightclient cometbls --lightclient state-lens/ics23/ics23 --lightclient state-lens/ics23/mpt \
               ${pkgs.lib.optionalString (u != null) "--u ${u}"} \
               ${pkgs.lib.optionalString (eu != null) "--eu ${eu}"} \
              --ucs03 "$@"
          '';
        };

      deploy =
        {
          name,

          chain-id,
          rpc-url,
          private-key,
          weth,
          rate-limit-enabled ? "true",
          native-token-name ? "Ether",
          native-token-symbol ? "ETH",
          native-token-decimals ? 18,

          verify ? true,
          verifier ? if verify then throw "verifier must be set in order to verify" else "",
          verification-key ? if verify then throw "verification-key must be set in order to verify" else "",
          verifier-url ? if verify then throw "verifier-url must be set in order to verify" else "",
          ...
        }:
        mkCi false (
          pkgs.writeShellApplicationWithArgs {
            name = "eth-deploy-${name}";
            runtimeInputs = [ self'.packages.forge ];
            arguments = [
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
              RATE_LIMIT_ENABLED=${rate-limit-enabled} \
              NATIVE_TOKEN_NAME=${native-token-name} \
              NATIVE_TOKEN_SYMBOL=${native-token-symbol} \
              NATIVE_TOKEN_DECIMALS=${toString native-token-decimals} \
              PRIVATE_KEY=''${PRIVATE_KEY:-${private-key}} \
              DEPLOYER="$argc_deployer_pk" \
              SENDER="$argc_sender_pk" \
              FOUNDRY_LIBS='["libs"]' \
              FOUNDRY_PROFILE="script" \
                forge script scripts/Deploy.s.sol:DeployIBC \
                -vvvv \
                --rpc-url "''${RPC_URL:-${rpc-url}}" \
                --broadcast "''${VERIFICATION_ARGS[@]}"

              popd
              rm -rf "$OUT"
            '';
          }
        );

      whitelist-relayers =
        {
          name,

          ucs04-chain-id,
          rpc-url,
          private-key,

          ...
        }:
        mkCi false (
          pkgs.writeShellApplication {
            name = "whitelist-relayers-${name}";
            runtimeInputs = [ pkgs.foundry-bin ];
            text = ''
              echo "whitelisting $# relayers"
              for relayer in "$@"
              do
                cast \
                  send \
                  ${getDeployedContractAddress ucs04-chain-id "manager"} \
                  "function grantRole(uint64,address,uint32)" \
                  1 "$relayer" 0 \
                  --private-key ${private-key} \
                  --rpc-url "''${RPC_URL:-${rpc-url}}"

                echo "whitelisted relayer $relayer"
              done
            '';
          }
        );

      set-bucket-config =
        {
          name,

          ucs04-chain-id,
          rpc-url,
          private-key,

          ...
        }:
        mkCi false (
          pkgs.writeShellApplicationWithArgs {
            name = "set-bucket-config-${name}";
            arguments = [
              {
                arg = "denom";
                required = true;
                help = "Denom to set bucket config for";
              }
              {
                arg = "capacity";
                help = "Capacity of the bucket";
              }
              {
                arg = "refill_rate";
                help = "Refill rate of the bucket";
              }
              # TODO: Figure out how to set --reset
            ];
            runtimeInputs = [ pkgs.foundry-bin ];
            text = ''
              echo "setting bucket config for $argc_denom"
              cast \
                send \
                ${(getDeployment ucs04-chain-id).app.ucs03.address} \
                "function setBucketConfig(address,uint256,uint256,bool)" \
                "$argc_denom" \
                "$argc_capacity" \
                "$argc_refill_rate" \
                false \
                --private-key ${private-key} \
                --rpc-url "''${RPC_URL:-${rpc-url}}"

              echo "set bucket config for $argc_denom"
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
          rate-limit-enabled ? "true",
          native-token-name ? "Ether",
          native-token-symbol ? "ETH",
          native-token-decimals ? 18,

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
              RATE_LIMIT_ENABLED=${rate-limit-enabled} \
              NATIVE_TOKEN_NAME=${native-token-name} \
              NATIVE_TOKEN_SYMBOL=${native-token-symbol} \
              NATIVE_TOKEN_DECIMALS=${toString native-token-decimals} \
              PRIVATE_KEY=''${PRIVATE_KEY:-${private-key}} \
              FOUNDRY_LIBS='["libs"]' \
              FOUNDRY_PROFILE="script" \
                forge script scripts/Deploy.s.sol:DeployDeployerAndIBC \
                -vvvv \
                --rpc-url "''${RPC_URL:-${rpc-url}}" \
                --broadcast "''${VERIFICATION_ARGS[@]}"

              popd
              rm -rf "$OUT"
            '';
          }
        );

      verify =
        {
          chain-id,
          rpc-url,

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

              OUT="$(mktemp -d)"
              pushd "$OUT"
              cp --no-preserve=mode -r ${self'.packages.evm-contracts}/* .
              cp --no-preserve=mode -r ${evmSources}/* .

              # shellcheck disable=SC2005
              FOUNDRY_ETHERSCAN="$FOUNDRY_ETHERSCAN" \
              VERIFICATION_KEY=${verification-key} \
              FOUNDRY_LIBS='["libs"]' \
                forge verify-contract \
                  --force \
                  --watch "$1" "$2" \
                  --rpc-url "''${RPC_URL:-${rpc-url}}" "''${VERIFICATION_ARGS[@]}"

              popd
              rm -rf "$OUT"
            '';
          }
        );

      # NOTE: Must verify against the implementation if verifying a proxy contract
      # the implementation can be fetched with: `cast impl $ADDRESS -r $RPC_URL`
      verify-against-commit =
        {
          chain-id,
          rpc-url,

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

              OUT="$(mktemp -d)"
              pushd "$OUT"

              echo "fetching sources at $3..."

              nix build "github:unionlabs/union/$3#evm-contracts"
              cp --no-preserve=mode -r result/* .

              nix build "github:unionlabs/union/$3#evm-contracts.out.src"
              cp --no-preserve=mode -r result/* .

              # shellcheck disable=SC2005
              FOUNDRY_ETHERSCAN="$FOUNDRY_ETHERSCAN" \
              VERIFICATION_KEY=${verification-key} \
              FOUNDRY_LIBS='["libs"]' \
                forge verify-bytecode \
                  "$1" "$2" \
                  --rpc-url "''${RPC_URL:-${rpc-url}}" "''${VERIFICATION_ARGS[@]}"

              popd
              rm -rf "$OUT"
            '';
          }
        );

      verify-erc20 =
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

              OUT="$(mktemp -d)"
              pushd "$OUT"
              cp --no-preserve=mode -r ${self'.packages.evm-contracts}/* .
              cp --no-preserve=mode -r ${evmSources}/* .

              # shellcheck disable=SC2005
              FOUNDRY_ETHERSCAN="$FOUNDRY_ETHERSCAN" \
              VERIFICATION_KEY=${verification-key} \
              FOUNDRY_LIBS='["libs"]' \
                forge verify-contract \
                  --force \
                  --watch "$1" "libs/@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy" \
                  --constructor-args "0x00" \
                  --rpc-url "''${RPC_URL:-${rpc-url}}" "''${VERIFICATION_ARGS[@]}"

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
          rate-limit-enabled ? "true",
          native-token-name ? "Ether",
          native-token-symbol ? "ETH",
          native-token-decimals ? 18,

          verify ? true,
          verifier ? if verify then throw "verifier must be set in order to verify" else "",
          verification-key ? if verify then throw "verification-key must be set in order to verify" else "",
          verifier-url ? if verify then throw "verifier-url must be set in order to verify" else "",
          ...
        }:
        mkCi false (
          pkgs.writeShellApplication {
            name = "eth-verify";
            runtimeInputs = [
              wrappedForgeOnline
              pkgs.jq
            ];
            text = ''
              ${ensureAtRepositoryRoot}

              ${setupFoundryVerifcationVars verify {
                inherit chain-id verifier verifier-url;
                with-verify-flag = false;
              }}

              WETH_ADDRESS=${weth} \
              RATE_LIMIT_ENABLED=${rate-limit-enabled} \
              NATIVE_TOKEN_NAME=${native-token-name} \
              NATIVE_TOKEN_SYMBOL=${native-token-symbol} \
              NATIVE_TOKEN_DECIMALS=${toString native-token-decimals} \
                nix run .#evm-contracts-addresses -- "$1" "$2" "''${RPC_URL:-${rpc-url}}"

              PROJECT_ROOT=$(pwd)
              OUT="$(mktemp -d)"
              pushd "$OUT"
              cp --no-preserve=mode -r ${self'.packages.evm-contracts}/* .
              cp --no-preserve=mode -r ${evmSources}/* .


              jq -r 'to_entries | map([.key, .value.args, .value.contract]) | .[] | @tsv' "$PROJECT_ROOT"/contracts.json | \
                while IFS=$'\t' read -r address args contract; do
                  if [ "$address" != "0x0000000000000000000000000000000000000000" ]
                  then
                    echo
                    echo "======================================================"
                    echo " Verifying $address "
                    echo "======================================================"
                    echo
                    # shellcheck disable=SC2005
                    FOUNDRY_ETHERSCAN="$FOUNDRY_ETHERSCAN" \
                    VERIFICATION_KEY=${verification-key} \
                    WETH_ADDRESS=${weth} \
                    RATE_LIMIT_ENABLED=${rate-limit-enabled} \
                    NATIVE_TOKEN_NAME=${native-token-name} \
                    NATIVE_TOKEN_SYMBOL=${native-token-symbol} \
                    NATIVE_TOKEN_DECIMALS=${toString native-token-decimals} \
                    PRIVATE_KEY=''${PRIVATE_KEY:-${private-key}} \
                    FOUNDRY_LIBS='["libs"]' \
                    FOUNDRY_PROFILE="script" \
                      forge verify-contract \
                        --force \
                        --watch "$address" "$contract" \
                        --constructor-args "$args" \
                        --rpc-url "''${RPC_URL:-${rpc-url}}" "''${VERIFICATION_ARGS[@]}" || true
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
          private-key,
          rpc-url,
          weth,
          rate-limit-enabled ? "true",
          native-token-name ? "Ether",
          native-token-symbol ? "ETH",
          native-token-decimals ? 18,

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
              RATE_LIMIT_ENABLED=${rate-limit-enabled} \
              NATIVE_TOKEN_NAME=${native-token-name} \
              NATIVE_TOKEN_SYMBOL=${native-token-symbol} \
              NATIVE_TOKEN_DECIMALS=${toString native-token-decimals} \
              DEPLOYER="$argc_deployer_pk" \
              SENDER="$argc_sender_pk" \
              PRIVATE_KEY=''${PRIVATE_KEY:-${private-key}} \
              FOUNDRY_LIBS='["libs"]' \
              FOUNDRY_PROFILE="script" \
                forge script scripts/Deploy.s.sol:Deploy${kind} \
                -vvvv \
                --rpc-url "''${RPC_URL:-${rpc-url}}" \
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
          safe ? false,
          protocol,

          ucs04-chain-id,
          chain-id,
          private-key,
          rpc-url,
          weth,
          rate-limit-enabled ? "true",
          native-token-name ? "Ether",
          native-token-symbol ? "ETH",
          native-token-decimals ? 18,

          verify ? true,
          verifier ? if verify then throw "verifier must be set in order to verify" else "",
          verification-key ? if verify then throw "verification-key must be set in order to verify" else "",
          verifier-url ? if verify then throw "verifier-url must be set in order to verify" else "",
          ...
        }:
        mkCi false (
          pkgs.writeShellApplicationWithArgs {
            name = "evm-${pkgs.lib.optionalString safe "safe"}${pkgs.lib.optionalString dry "dry"}upgrade-${protocol}";
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
              ++ pkgs.lib.optionals dry [
                {
                  arg = "owner_pk";
                  required = true;
                  help = "The contract owner public key to prank.";
                }
                {
                  arg = "dry_url";
                  required = true;
                  help = "The rpc url to use for dry running the tx.";
                }
              ];
            text = ''
              OUT="$(mktemp -d)"
              pushd "$OUT"
              cp --no-preserve=mode -r ${self'.packages.evm-contracts}/* .
              cp --no-preserve=mode -r ${evmSources}/* .

              ${setupFoundryVerifcationVars verify {
                inherit chain-id verifier verifier-url;
                with-verify-flag = false;
              }}

              UNIVERSAL_CHAIN_ID=${ucs04-chain-id} \
              WETH_ADDRESS=${weth} \
              RATE_LIMIT_ENABLED=${rate-limit-enabled} \
              NATIVE_TOKEN_NAME=${native-token-name} \
              NATIVE_TOKEN_SYMBOL=${native-token-symbol} \
              NATIVE_TOKEN_DECIMALS=${toString native-token-decimals} \
              FOUNDRY_ETHERSCAN="$FOUNDRY_ETHERSCAN" \
              VERIFICATION_KEY=${verification-key} \
              DEPLOYER="$argc_deployer_pk" \
              SENDER="$argc_sender_pk" \
              OWNER="${pkgs.lib.optionalString dry "$argc_owner_pk"}" \
              PRIVATE_KEY=''${PRIVATE_KEY:-${private-key}} \
              FOUNDRY_LIBS='["libs"]' \
              FOUNDRY_PROFILE="script" \
                forge script scripts/Deploy.s.sol:${pkgs.lib.optionalString safe "Safe"}${pkgs.lib.optionalString dry "Dry"}Upgrade${protocol} -vvvvv \
                  --slow \
                  --rpc-url ${if dry then "$argc_dry_url" else rpc-url} \
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
                  ${contracts}/out/Zkgm.sol/UCS03Zkgm.json > protocols-ucs03.json

                jq --compact-output --slurp 'map(.abi) | add' \
                  ${contracts}/out/UDrop.sol/UDrop.json > udrop.json

                jq --compact-output --slurp 'map(.abi) | add' \
                  ${contracts}/out/CometblsClient.sol/CometblsClient.json \
                  ${contracts}/out/CometblsClient.sol/CometblsClientLib.json > lightclients-cometbls.json

                jq --compact-output --slurp 'map(.abi) | add' \
                  ${contracts}/out/StateLensIcs23MptClient.sol/StateLensIcs23MptClient.json \
                  ${contracts}/out/StateLensIcs23MptClient.sol/StateLensIcs23MptLib.json > lightclients-state-lens-ics23-mpt.json

                jq --compact-output --slurp 'map(.abi) | add' \
                  ${contracts}/out/StateLensIcs23Ics23Client.sol/StateLensIcs23Ics23Client.json \
                  ${contracts}/out/StateLensIcs23Ics23Client.sol/StateLensIcs23Ics23Lib.json > lightclients-state-lens-ics23-ics23.json

                jq --compact-output --slurp 'map(.abi) | add' \
                  ${contracts}/out/StateLensIcs23SmtClient.sol/StateLensIcs23SmtClient.json \
                  ${contracts}/out/StateLensIcs23SmtClient.sol/StateLensIcs23SmtLib.json > lightclients-state-lens-ics23-smt.json

                jq --compact-output --slurp 'map(.abi) | add' \
                  ${contracts}/out/ProofLensClient.sol/ProofLensClient.json \
                  ${contracts}/out/ProofLensClient.sol/ProofLensLib.json > lightclients-proof-lens.json

                jq --compact-output --slurp 'map(.abi) | add' \
                  ${contracts}/out/LoopbackClient.sol/LoopbackClient.json \
                  ${contracts}/out/LoopbackClient.sol/LoopbackClientLib.json > lightclients-loopback.json
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
          {
            update-deployments-json = pkgs.writeShellApplication {
              name = "update-deployments-json";
              text =
                let
                  deployments = pkgs.lib.filterAttrs (_: deployment: deployment.ibc_interface == "ibc-solidity") (
                    builtins.fromJSON (builtins.readFile ../deployments/deployments.json)
                  );
                  getNetwork =
                    ucs04chainId:
                    pkgs.lib.lists.findSingle (network: network.ucs04-chain-id or null == ucs04chainId)
                      (throw "no network found with ucs04 chain id ${ucs04chainId}")
                      (throw "many networks with ucs04 chain id ${ucs04chainId} found")
                      networks;
                in
                pkgs.lib.concatMapStringsSep "\n\n" (ucs04ChainId: ''
                  echo "updating ${ucs04ChainId}"
                  ${pkgs.lib.getExe
                    self'.packages.evm-scripts.${(getNetwork ucs04ChainId).name}.update-deployments-json
                  }
                '') (builtins.attrNames deployments);
            };
          }
          // (builtins.listToAttrs (
            map (chain: {
              inherit (chain) name;
              value = pkgs.mkRootDrv chain.name (
                {
                  verify = verify chain;
                  verify-all-contracts = verify-all-contracts chain;
                  verify-erc20 = verify-erc20 chain;
                  deploy = deploy chain;
                  deploy-deployer-and-ibc = deploy-deployer-and-ibc chain;
                  update-deployments-json = update-deployments-json chain;
                  whitelist-relayers = whitelist-relayers chain;
                  set-bucket-config = set-bucket-config chain;
                  verify-against-commit = verify-against-commit chain;
                  # finalize-deployment = finalize-deployment chain;
                  # get-git-rev = get-git-rev chain;
                }
                # individual deployments
                // (pkgs.lib.mapAttrs'
                  (name: kind: {
                    name = "deploy-${name}";
                    value = deploy-single (chain // { inherit kind; });
                  })
                  {
                    ucs03 = "UCS03";
                    cometbls-client = "CometblsClient";
                    loopback-client = "LoopbackClient";
                    state-lens-ics23-mpt-client = "StateLensIcs23MptClient";
                    state-lens-ics23-ics23-client = "StateLensIcs23Ics23Client";
                    state-lens-ics23-smt-client = "StateLensIcs23SmtClient";
                    proof-lens-client = "ProofLensClient";
                    multicall = "Multicall";
                    erc20 = "ZkgmERC20";
                    proxy-account = "ProxyAccount";
                    u = "U";
                    eu = "EU";
                    udrop = "UDrop";
                    eudrop = "EUDrop";
                    quick-withdrawal = "QuickWithdrawal";
                    z-asset = "ZAsset";
                    unionversal-token = "UnionversalToken";
                  }
                )
                # other various deployment scripts
                // (pkgs.lib.mapAttrs'
                  (name: kind: {
                    name = "script-${name}";
                    value = deploy-single (chain // { inherit kind; });
                  })
                  {
                    roles = "Roles";
                    register-clients = "RegisterClients";
                  }
                )
                # upgrades, all with a -dry version
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
                        ++ [
                          {
                            ${"safe-upgrade-${name}"} = upgrade (
                              chain
                              // {
                                inherit protocol;
                                dry = false;
                                safe = true;
                              }
                            );
                          }
                        ]
                      )
                      {
                        ucs00 = "UCS00";
                        ucs03 = "UCS03";
                        ucs03-v1-to-v2 = "UCS03FromV1ToV2";

                        cometbls-client = "CometblsClient";
                        state-lens-ics23-mpt-client = "StateLensIcs23MptClient";
                        state-lens-ics23-ics23-client = "StateLensIcs23Ics23Client";
                        state-lens-ics23-smt-client = "StateLensIcs23SmtClient";
                        proof-lens-client = "ProofLensClient";
                        core = "IBCHandler";
                        u = "U";
                        eu = "EU";
                        udrop = "UDrop";
                        eudrop = "EUDrop";
                        quick-withdrawal = "QuickWithdrawal";
                        z-asset = "ZAsset";
                        loopback-client = "LoopbackClient";
                        unionversal-token = "UnionversalToken";
                      }
                  )
                ))
              );
            }) networks
          ))
        );
      };
    };
}
