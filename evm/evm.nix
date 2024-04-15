{ ... }: {
  perSystem = { self', pkgs, proto, nix-filter, ensureAtRepositoryRoot, system, mkCi, ... }:
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
        rev = "v0.0.162";
        hash = "sha256-9lgXwW2YQABfaklGdDYIXU8qFBapszoB4+mAatKV9bs=";
      };
      forge-std = pkgs.fetchFromGitHub {
        owner = "foundry-rs";
        repo = "forge-std";
        rev = "v1.8.1";
        hash = "sha256-s/J7odpWysj4U93knIRA28aZqXworZH6IVIjIXD78Yc=";
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
      linkedLibs = pkgs.linkFarm "evm-libraries" [
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
      libraries = pkgs.stdenv.mkDerivation {
        name = "libraries";
        phases = [ "installPhase" "fixupPhase" ];
        src = "${linkedLibs}";
        installPhase = ''
          mkdir $out
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
      evmSources = nix-filter {
        root = ./.;
        include = [
          "contracts"
          "tests"
        ];
      };
      # Foundry FS permissions must be explicitly set in the config file
      foundryConfig = pkgs.writeTextDir "/foundry.toml" ''
        [profile.default]
        fs_permissions = [{ access = "read", path = "./"}]
        libs = ["${libraries}"]
        gas_reports = ["*"]
        via_ir = true

        [profile.optimized]
        src = "${evmSources}/contracts"
        optimizer = true
        optimizer_runs = 10_000_000

        [profile.script]
        src = "${evmSources}/scripts"
        optimizer = true
        optimizer_runs = 10_000_000

        [profile.test]
        test = "${evmSources}/tests/src"
        optimizer = false
        ast = true
      '';
      wrappedForge = pkgs.symlinkJoin {
        name = "forge";
        paths = [ pkgs.foundry-bin ];
        buildInputs = [ pkgs.makeWrapper ];
        postBuild = ''
          wrapProgram $out/bin/forge \
            --append-flags "--offline --no-auto-detect" \
            --set PATH ${pkgs.lib.makeBinPath [ pkgs.solc ]} \
            --set SSL_CERT_FILE "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" \
            --set FOUNDRY_CONFIG "${foundryConfig}/foundry.toml"
        '';
      };
      networks = [
        {
          network = "devnet";
          rpc-url = "http://localhost:8545";
          private-key = builtins.readFile ./../networks/genesis/devnet-eth/dev-key0.prv;
        }
        {
          network = "testnet";
          rpc-url = "https://rpc-sepolia.rockx.com";
          private-key = ''"$1"'';
        }
        {
          network = "scroll-testnet";
          rpc-url = "https://sepolia-rpc.scroll.io";
          private-key = ''"$1"'';
        }
      ];

      deploy-contracts = { rpc-url, private-key }: contracts:
        pkgs.lib.concatStrings (pkgs.lib.forEach contracts (contract:
          deploy {
            inherit rpc-url private-key;
            inherit (contract) path name;
            args = contract.args or "";
          }));

      deploy = { rpc-url, private-key, path, name, args ? "" }: ''
        echo "Deploying ${name}..."
        ${pkgs.lib.toUpper name}=$(FOUNDRY_PROFILE=optimized forge create \
                 --json \
                 --rpc-url ${rpc-url} \
                 --private-key ${private-key} \
                 ${evmSources}/contracts/${path}:${name} ${args} | jq --raw-output .deployedTo)
        echo "${name} => ''$${pkgs.lib.toUpper name}"
      '';

      deploy-ibc-contracts = { network, rpc-url, private-key }:
        mkCi false (pkgs.writeShellApplication {
          name = "eth-${network}-deploy";
          runtimeInputs = [ pkgs.jq wrappedForge ];
          # Sadly, forge is trying to write back the cache file even if no change is needed :).
          # For this reason we copy the artifacts in a temp folder and work from there.
          text = ''
            OUT="$(mktemp -d)"
            cd "$OUT"
            cp --no-preserve=mode -r ${self'.packages.evm-contracts}/* .

            ${deploy-contracts { inherit rpc-url private-key; } [
              { path = "core/02-client/IBCClient.sol"; name = "IBCClient"; }
              { path = "core/03-connection/IBCConnection.sol"; name = "IBCConnection"; }
              { path = "core/04-channel/IBCChannelHandshake.sol"; name = "IBCChannelHandshake"; }
              { path = "core/04-channel/IBCPacket.sol"; name = "IBCPacket"; }
              { path = "core/DevnetIBCHandlerInit.sol"; name = "DevnetIBCHandlerInit"; }
              { path = "core/DevnetOwnableIBCHandler.sol"; name = "DevnetOwnableIBCHandler"; args = ''--constructor-args "$IBCCLIENT" "$IBCCONNECTION" "$IBCCHANNELHANDSHAKE" "$IBCPACKET" "$DEVNETIBCHANDLERINIT"''; }

              { path = "clients/CometblsClientV2.sol"; name = "CometblsClient"; args = ''--constructor-args "$DEVNETOWNABLEIBCHANDLER"''; }

              { path = "apps/ucs/01-relay/Relay.sol"; name = "UCS01Relay"; args = ''--constructor-args "$DEVNETOWNABLEIBCHANDLER" "1"'';}
            ]}

            echo "{\"ibc_handler_address\": \"$DEVNETOWNABLEIBCHANDLER\", \"cometbls_client_address\": \"$COMETBLSCLIENT\", \"ucs01_relay_address\": \"$UCS01RELAY\"  }"

            rm -rf "$OUT"
          '';
        });

      deploy-ping-pong = { network, rpc-url, private-key, ... }: mkCi false (pkgs.writeShellApplication {
        name = "evm-${network}-ping-pong-deploy";
        runtimeInputs = [ pkgs.jq wrappedForge ];
        text = ''
          OUT="$(mktemp -d)"
          cd "$OUT"
          cp --no-preserve=mode -r ${self'.packages.evm-contracts}/* .

          ${deploy-contracts { rpc-url = rpc-url;
                           private-key = private-key; } [{
                           path = "apps/ucs/00-pingpong/PingPong.sol";
                           name = "PingPong";
                           args = ''--constructor-args "$IBC_HANDLER_ADDRESS" "$REVISION_NUMBER" "$NUM_OF_BLOCK_BEFORE_PONG_TIMEOUT" ''; }]}

          echo "{\"ping_pong_address\": \"$PINGPONG\" }"

          rm -rf "$OUT"
        '';
      });

      deploy-ucs01 = { network, rpc-url, private-key, ... }: mkCi false (pkgs.writeShellApplication {
        name = "evm-${network}-ucs01";
        runtimeInputs = [ pkgs.jq wrappedForge ];
        text = ''
          OUT="$(mktemp -d)"
          cd "$OUT"
          cp --no-preserve=mode -r ${self'.packages.evm-contracts}/* .

          ${deploy-contracts { rpc-url = rpc-url;
                           private-key = private-key; } [{
                           path = "apps/ucs/01-relay/Relay.sol";
                           name = "UCS01Relay";
                           args = ''--constructor-args "$IBC_HANDLER_ADDRESS"''; }]}

          echo "{\"ucs01_address\": \"$UCS01RELAY\" }"

          rm -rf "$OUT"
        '';
      });

      deploy-ucs02 = { network, rpc-url, private-key, ... }: mkCi false (pkgs.writeShellApplication {
        name = "evm-${network}-ucs02";
        runtimeInputs = [ pkgs.jq wrappedForge ];
        text = ''
          OUT="$(mktemp -d)"
          cd "$OUT"
          cp --no-preserve=mode -r ${self'.packages.evm-contracts}/* .

          ${deploy-contracts { rpc-url = rpc-url;
                           private-key = private-key; } [{
                           path = "apps/ucs/02-nft/NFT.sol";
                           name = "UCS02NFT";
                           args = ''--constructor-args "$IBC_HANDLER_ADDRESS"''; }]}

          echo "{\"ucs02_address\": \"$UCS02NFT\" }"

          rm -rf "$OUT"
        '';
      });
    in
    {
      packages = {
        # Beware, the generate solidity code is broken and require manual patch. Do not update unless you know that aliens exists.
        generate-sol-proto = mkCi false (pkgs.writeShellApplication {
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
                buildInputs =
                  [ (pkgs.python3.withPackages (ps: with ps; [ protobuf wrapt ])) ];
                buildPhase = "true";
                installPhase = ''
                  mkdir $out
                  cp -r $src/* $out
                '';
              };
              protoIncludes = ''
                -I"${proto.cometbls}/proto" -I"${proto.cosmossdk}/proto" -I"${proto.ibc-go}/proto" -I"${proto.cosmosproto}/proto" -I"${proto.ics23}/proto" -I"${proto.googleapis}" -I"${proto.gogoproto}" -I"${proto.uniond}"'';
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
        });

        evm-contracts = mkCi (system == "x86_64-linux") (pkgs.stdenv.mkDerivation {
          name = "evm-contracts";
          src = evmSources;
          buildInputs = [ wrappedForge ];
          buildPhase = ''
            forge --version
            FOUNDRY_PROFILE=optimized forge build --sizes
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
        });

        external-evm-contracts = mkCi (system == "x86_64-linux") (pkgs.stdenv.mkDerivation {
          name = "external-evm-contracts";
          src = "${openzeppelin}/contracts/token/ERC20";
          buildInputs = [ wrappedForge ];
          buildPhase = ''
            forge --version
            cp ${foundryConfig}/foundry.toml .
            FOUNDRY_PROFILE=optimized forge build
          '';
          doCheck = false;
          installPhase = ''
            mkdir -p $out
            mv out $out
            mv cache $out
          '';
        });

        # NOTE: currently unable to build the tests with coverage, tried many different combination of the optimizer though...
        # solidity-coverage =
        #   pkgs.runCommand "solidity-coverage"
        #     {
        #       buildInputs = [ self'.packages.forge pkgs.lcov ];
        #     } ''
        #     FOUNDRY_PROFILE="test" forge coverage --ir-minimum --report lcov
        #     lcov --remove ./lcov.info -o ./lcov.info.pruned \
        #       '${evmSources}/contracts/proto/*' \
        #       '${evmSources}/contracts/clients/MockClient.sol' \
        #       '${evmSources}/contracts/clients/Verifier.sol' \
        #       '${evmSources}/contracts/apps/ucs/00-pingpong/*' \
        #       '${evmSources}/contracts/core/DevnetIBCHandlerInit.sol' \
        #       '${evmSources}/contracts/core/DevnetOwnableIBCHandler.sol' \
        #       '${evmSources}/contracts/core/OwnableIBCHandler.sol' \
        #       '${evmSources}/contracts/core/25-handler/IBCQuerier.sol' \
        #       '${evmSources}/contracts/core/24-host/IBCCommitment.sol' \
        #       '${evmSources}/tests/*'
        #     genhtml lcov.info.pruned -o $out --branch-coverage
        #     mv lcov.info.pruned $out/lcov.info
        #   '';
        # show-solidity-coverage = pkgs.writeShellApplication {
        #   name = "show-solidity-coverage";
        #   runtimeInputs = [ ];
        #   text = ''
        #     xdg-open ${self'.packages.solidity-coverage}/index.html
        #   '';
        # };

        solidity-build-tests = pkgs.writeShellApplication {
          name = "run-solidity-build-tests";
          runtimeInputs = [ self'.packages.forge ];
          text = ''
            ${ensureAtRepositoryRoot}
            FOUNDRY_PROFILE="test" FOUNDRY_TEST="evm/tests/src" forge test -vvv --gas-report
          '';
        };

        eth-deploy-deployer = pkgs.writeShellApplication {
          name = "deploy-deployer";
          runtimeInputs = [ self'.packages.forge ];
          text = ''
            ${ensureAtRepositoryRoot}
            PRIVATE_KEY=0x${builtins.readFile ./../networks/genesis/devnet-eth/dev-key0.prv} FOUNDRY_PROFILE="script" forge script evm/scripts/Deploy.s.sol:DeployDeployer -vvv --rpc-url http://localhost:8545 --broadcast
          '';
        };

        eth-deploy-stack = pkgs.writeShellApplication {
          name = "deploy-stack";
          runtimeInputs = [ self'.packages.forge ];
          text = ''
            ${ensureAtRepositoryRoot}
            PRIVATE_KEY=0x${builtins.readFile ./../networks/genesis/devnet-eth/dev-key0.prv} FOUNDRY_PROFILE="script" forge script evm/scripts/Deploy.s.sol:DeployIBC -vvv --rpc-url http://localhost:8545 --broadcast
          '';
        };

        forge = wrappedForge;
      } //
      builtins.listToAttrs (
        builtins.map
          (args: { name = "eth-${args.network}-deploy"; value = deploy-ibc-contracts args; })
          networks
      ) //
      builtins.listToAttrs (
        builtins.map
          (args: { name = "eth-${args.network}-ping-pong-deploy"; value = deploy-ping-pong args; })
          networks
      ) //
      builtins.listToAttrs (
        builtins.map
          (args: { name = "eth-${args.network}-ucs01-deploy"; value = deploy-ucs01 args; })
          networks
      ) //
      builtins.listToAttrs (
        builtins.map
          (args: { name = "eth-${args.network}-ucs02-nft"; value = deploy-ucs02 args; })
          networks
      );
    };
}
