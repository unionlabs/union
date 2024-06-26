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
        phases = [ "installPhase" "fixupPhase" ];
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
        phases = [ "installPhase" "fixupPhase" ];
        src = evmLibs;
        installPhase = ''
          mkdir -p $out/libs
          cp -rL $src/* $out/libs
          cp -r ${nix-filter {
            root = ./.;
            include = [
              "scripts"
              "contracts"
              "tests"
            ];
          }}/* $out/
        '';
      };
      # Foundry FS permissions must be explicitly set in the config file
      foundryConfig = pkgs.writeTextDir "/foundry.toml" ''
        [profile.default]
        fs_permissions = [{ access = "read", path = "./"}]
        libs = ["libs"]
        gas_reports = ["*"]
        via_ir = true

        [profile.script]
        src = "scripts"
        bytecode_hash = "none"
        cbor_metadata = false
        sparse_mode = false
        optimizer = true
        optimizer_runs = 10_000_000

        [profile.test]
        test = "tests/src"
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
          network = "testnet";
          rpc-url = "https://rpc-sepolia.rockx.com";
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

      eth-deploy = { rpc-url, private-key, extra-args ? "", ... }: pkgs.writeShellApplication {
        name = "eth-deploy";
        runtimeInputs = [ self'.packages.forge ];
        text = ''
          ${ensureAtRepositoryRoot}
          OUT="$(mktemp -d)"
          pushd "$OUT"
          cp --no-preserve=mode -r ${self'.packages.evm-contracts}/* .
          cp --no-preserve=mode -r ${evmSources}/* .

          PRIVATE_KEY=${private-key} FOUNDRY_PROFILE="script" \
            forge script scripts/Deploy.s.sol:DeployDeployerAndIBC \
            -vvvv \
            --rpc-url ${rpc-url} \
            --broadcast ${extra-args}

          popd
          rm -rf "$OUT"
        '';
      };
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
            FOUNDRY_PROFILE=script forge build
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

        hubble-abis =
          let
            contracts = self'.packages.evm-contracts;
          in
          pkgs.runCommand "hubble-abis"
            {
              buildInputs = [ pkgs.jq ];
            } ''
            mkdir -p $out
            cd $out

            jq --compact-output --slurp 'map(.abi) | add' \
              ${contracts}/out/IBCClient.sol/IBCClient.json \
              ${contracts}/out/IBCPacket.sol/IBCPacket.json \
              ${contracts}/out/IBCConnection.sol/IBCConnection.json \
              ${contracts}/out/OwnableIBCHandler.sol/OwnableIBCHandler.json \
              ${contracts}/out/IBCChannelHandshake.sol/IBCChannelHandshake.json > ibc-handler.json 

            jq --compact-output --slurp 'map(.abi) | add' \
              ${contracts}/out/Relay.sol/IRelay.json \
              ${contracts}/out/Relay.sol/UCS01Relay.json \
              ${contracts}/out/Relay.sol/RelayLib.json \
              ${contracts}/out/Relay.sol/RelayPacketLib.json > ucs-01.json 

            jq --compact-output --slurp 'map(.abi) | add' \
              ${contracts}/out/NFT.sol/NFTLib.json \
              ${contracts}/out/NFT.sol/NFTPacketLib.json \
              ${contracts}/out/NFT.sol/UCS02NFT.json > ucs-02.json 
          '';


        solidity-build-tests = pkgs.writeShellApplication {
          name = "run-solidity-build-tests";
          runtimeInputs = [ self'.packages.forge ];
          text = ''
            ${ensureAtRepositoryRoot}
            FOUNDRY_LIBS=["${evmLibs}"] FOUNDRY_PROFILE="test" FOUNDRY_TEST="evm/tests/src" forge test -vvv --gas-report "$@"
          '';
        };

        evm-contracts-addresses = pkgs.writeShellApplication {
          name = "eth-contracts-addresses";
          runtimeInputs = [ self'.packages.forge pkgs.jq ];
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
        };

        forge = wrappedForge;

        evm-sources = evmSources;
      } //
      builtins.listToAttrs (
        builtins.map
          (args: { name = "eth-deploy-${args.network}"; value = eth-deploy args; })
          networks
      );
    };
}
