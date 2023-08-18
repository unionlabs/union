{ ... }: {
  perSystem = { self', inputs', pkgs, proto, nix-filter, ... }:
    let
      solidity-stringutils = pkgs.fetchFromGitHub {
        owner = "Arachnid";
        repo = "solidity-stringutils";
        rev = "46983c6d9462a80229cf0d5bab8ea3b3ee31066c";
        hash = "sha256-8LGScZp29zOnXG8tXv62RHr+fJCWs0WbMpsZo9S95TE=";
      };
      solidity-bytes-utils = pkgs.fetchFromGitHub {
        owner = "GNSPS";
        repo = "solidity-bytes-utils";
        rev = "6458fb2780a3092bc756e737f246be1de6d3d362";
        hash = "sha256-sJWoYag6hTIoS4Jr1XdqBKfrJaFQ1iMPy+UI5vVb7Lw=";
      };
      solady = pkgs.fetchFromGitHub {
        owner = "vectorized";
        repo = "solady";
        rev = "e158762ba98db40a06411db7f80a54b93e951818";
        hash = "sha256-a5hiMUFQvE76h98md11+ksmmYsxV1p6t/ACO/hE2Cws=";
      };
      forge-std = pkgs.fetchFromGitHub {
        owner = "foundry-rs";
        repo = "forge-std";
        rev = "20872c5b1900526579159bdc6967f6b48c22e50e";
        hash = "sha256-dMfYen5EOUwjD6W1FY/vcNq2aN493rf7OfNlMfKnpwI=";
        fetchSubmodules = true;
      };
      openzeppelin = pkgs.fetchFromGitHub {
        owner = "OpenZeppelin";
        repo = "openzeppelin-contracts";
        rev = "v4.8.3";
        hash = "sha256-Qt2qC7T0gx18ydvO/UULEJj/q7ioGpNxJkT5el8hv14=";
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
          name = "ds-test";
          path = "${forge-std}/lib/ds-test/src";
        }
        {
          name = "@openzeppelin";
          path = "${openzeppelin}";
        }
      ];
      libraries = pkgs.stdenv.mkDerivation {
        name = "libraries";
        phases = [ "installPhase" ];
        installPhase = ''
          mkdir $out
          cp -rL ${linkedLibs}/* $out
        '';
      };
      evmSources = nix-filter {
        root = ./.;
        include = [
          "contracts"
          "tests"
        ];
      };
      foundryEnv = {
        FOUNDRY_OPTIMIZER = "true";
        FOUNDRY_VIA_IR = "true";
        FOUNDRY_OPTIMIZER_RUNS = "10000";
        FOUNDRY_SRC = "${evmSources}/contracts";
        FOUNDRY_TEST = "${evmSources}/tests/src";
        FOUNDRY_LIBS = ''["${libraries}"]'';
        FOUNDRY_GAS_REPORTS = ''["*"]'';
      };
      wrappedForge = pkgs.symlinkJoin {
        name = "forge";
        paths = [ pkgs.foundry-bin ];
        buildInputs = [ pkgs.makeWrapper ];
        postBuild = ''
          wrapProgram $out/bin/forge \
            --append-flags "--offline --no-auto-detect" \
            --set PATH ${pkgs.lib.makeBinPath [ pkgs.solc ]} \
            --set SSL_CERT_FILE "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" \
             ${pkgs.lib.foldlAttrs (acc: name: value: "${acc} --set-default ${name} '${value}'") "" foundryEnv}
        '';
      };
      networks = [
        {
          network = "devnet";
          rpc-url = "http://localhost:8545";
          private-key = builtins.readFile ./../networks/genesis/devnet-evm/dev-key0.prv;
        }
        {
          network = "testnet";
          rpc-url = "https://rpc.sepolia.org/";
          private-key = ''"$1"'';
        }
      ];

      deploy-contracts = { rpc-url, private-key }: contracts:
        pkgs.lib.concatStrings (pkgs.lib.forEach contracts (contract:
          deploy {
            inherit rpc-url private-key;
            inherit (contract) path name;
            create-args =
              if contract ? "optimize" then
                if contract.optimize then
                  "--optimize"
                else
                  ""
              else "--revert-strings debug";
            args = if contract ? "args" then contract.args else "";
          }));

      deploy = { rpc-url, private-key, path, name, args ? "", create-args ? "" }: ''
        echo "Deploying ${name}..."
        ${pkgs.lib.toUpper name}=$(forge create \
                 ${create-args} --json \
                 --rpc-url ${rpc-url} \
                 --private-key ${private-key} \
                 ${evmSources}/contracts/${path}:${name} ${args} | jq --raw-output .deployedTo)
        echo "${name} => ''$${pkgs.lib.toUpper name}"
      '';

      deploy-ibc-contracts = { network, rpc-url, private-key }:
        let
          # Upper first char of network
          verifierPrefix =
            pkgs.lib.strings.concatStrings (
              pkgs.lib.lists.imap0
                (i: c: if i == 0 then pkgs.lib.strings.toUpper c else c)
                (pkgs.lib.strings.stringToCharacters network));
        in
        pkgs.writeShellApplication {
          name = "evm-${network}-deploy";
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
              { path = "core/DevnetOwnableIBCHandler.sol"; name = "DevnetOwnableIBCHandler"; args = ''--constructor-args "$IBCCLIENT" "$IBCCONNECTION" "$IBCCHANNELHANDSHAKE" "$IBCPACKET"''; optimize = true; }

              { path = "clients/${verifierPrefix}Verifier.sol"; name = "${verifierPrefix}Verifier"; }
              { path = "clients/ICS23MembershipVerifier.sol"; name = "ICS23MembershipVerifier"; }
              { path = "clients/CometblsClient.sol"; name = "CometblsClient"; args = ''--constructor-args "$DEVNETOWNABLEIBCHANDLER" "''$${pkgs.lib.strings.toUpper network}VERIFIER" "$ICS23MEMBERSHIPVERIFIER"''; }

              { path = "apps/20-transfer/ICS20Bank.sol"; name = "ICS20Bank"; }
              { path = "apps/20-transfer/ICS20TransferBank.sol"; name = "ICS20TransferBank";  args = ''--constructor-args "$DEVNETOWNABLEIBCHANDLER" "$ICS20BANK"''; }              
            ]}

            echo "{\"ibc_handler_address\": \"$DEVNETOWNABLEIBCHANDLER\", \"cometbls_client_address\": \"$COMETBLSCLIENT\", \"ics20_transfer_bank_address\": \"$ICS20TRANSFERBANK\", \"ics20_bank_address\": \"$ICS20BANK\" }"

            rm -rf "$OUT"
          '';
        };

      deploy-ping-pong = { network, rpc-url, private-key }: pkgs.writeShellApplication {
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
      };
    in
    {
      packages = {
        # Beware, the generate solidity code is broken and require manual patch. Do not update unless you know that aliens exists.
        generate-evm-proto = pkgs.writeShellApplication {
          name = "generate-evm-proto";
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
                -I"${proto.cometbls}/proto" -I"${proto.cosmossdk}/proto" -I"${proto.ibcgo}/proto" -I"${proto.cosmosproto}/proto" -I"${proto.ics23}/proto" -I"${proto.googleapis}" -I"${proto.gogoproto}" -I"${proto.uniond}"'';
            in
            ''
              plugindir="${solidity-protobuf}/protobuf-solidity/src/protoc"
              # find ${proto.ibcgo}/proto -name "$1" |\
              # while read -r file; do
              #   echo "Generating $file"
              #   protoc \
              #     ${protoIncludes} \
              #    -I"$plugindir/include" \
              #    --plugin="protoc-gen-sol=$plugindir/plugin/gen_sol.py" \
              #    --sol_out=gen_runtime="ProtoBufRuntime.sol&solc_version=0.8.18:$2" \
              #     "$file"
              # done
              # find ${proto.cometbls}/proto -type f -regex ".*canonical.proto" |\
              # while read -r file; do
              #   echo "Generating $file"
              #   protoc \
              #     ${protoIncludes} \
              #    -I"$plugindir/include" \
              #    --plugin="protoc-gen-sol=$plugindir/plugin/gen_sol.py" \
              #    --sol_out=gen_runtime="ProtoBufRuntime.sol&solc_version=0.8.18:$2" \
              #     "$file"
              # done

              find ${proto.uniond} -type f -regex ".*ibc.*cometbls.*proto" |\
              while read -r file; do
                echo "Generating $file"
                protoc \
                  ${protoIncludes} \
                 -I"$plugindir/include" \
                 --plugin="protoc-gen-sol=$plugindir/plugin/gen_sol.py" \
                 --sol_out=gen_runtime="ProtoBufRuntime.sol&solc_version=0.8.18:$2" \
                  "$file"
              done
            '';
        };

        evm-contracts = pkgs.stdenv.mkDerivation {
          name = "evm-contracts";
          src = evmSources;
          buildInputs = [ wrappedForge ];
          buildPhase = ''
            forge build --revert-strings debug
          '';
          doCheck = true;
          checkPhase = ''
            forge --version
            forge test --revert-strings debug -vvv --gas-report
          '';
          installPhase = ''
            mkdir -p $out
            mv out $out
            mv cache $out
          '';
        };

        evm-coverage =
          pkgs.runCommand "evm-coverage.log"
            {
              buildInputs = [ wrappedForge pkgs.lcov ];
            } "forge coverage --ir-minimum --report lcov && genhtml lcov.info -o $out --branch-coverage";

        show-evm-coverage = pkgs.writeShellApplication {
          name = "show-evm-coverage";
          runtimeInputs = [ wrappedForge ];
          text = ''
            xdg-open ${self'.packages.evm-coverage}/index.html
          '';
        };
        forge = wrappedForge;
      } //
      builtins.listToAttrs (
        builtins.map
          (args: { name = "evm-${args.network}-deploy"; value = deploy-ibc-contracts args; })
          networks
      ) //
      builtins.listToAttrs (
        builtins.map
          (args: { name = "evm-${args.network}-ping-pong-deploy"; value = deploy-ping-pong args; })
          networks
      );
    };
}
