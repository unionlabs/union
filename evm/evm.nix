{ ... }: {
  perSystem = { self', pkgs, proto, forge, ... }:
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
      mkEvmContracts = { optimizerRuns, doCheck ? true }: pkgs.stdenv.mkDerivation {
        name = "evm-contracts";
        src = ./.;
        buildInputs = [ forge pkgs.solc ];
        buildPhase = ''
          forge build --offline --no-auto-detect
        '';
        inherit doCheck;
        checkPhase = ''
          forge test --offline --no-auto-detect
        '';
        installPhase = ''
          mkdir -p $out
          mv out/* $out
        '';
        FOUNDRY_SRC = "./contracts";
        FOUNDRY_TEST = "./tests/src";
        FOUNDRY_LIBS = ''["${libraries}"]'';
        FOUNDRY_OPTIMIZER = "true";
        FOUNDRY_VIA_IR = "true";
        FOUNDRY_OPTIMIZER_RUNS = builtins.toString optimizerRuns;
        FOUNDRY_GAS_REPORTS = ''["*"]'';
        SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
      };
    in
    {
      # Beware, the generate solidity code is broken and require manual patch. Do not update unless you know that aliens exists.
      packages.generate-evm-proto = pkgs.writeShellApplication {
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
            find ${proto.ibcgo}/proto -name "$1" |\
            while read -r file; do
              echo "Generating $file"
              protoc \
                ${protoIncludes} \
               -I"$plugindir/include" \
               --plugin="protoc-gen-sol=$plugindir/plugin/gen_sol.py" \
               --sol_out=gen_runtime="ProtoBufRuntime.sol&solc_version=0.8.18:$2" \
                "$file"
            done
            find ${proto.cometbls}/proto -type f -regex ".*canonical.proto" |\
            while read -r file; do
              echo "Generating $file"
              protoc \
                ${protoIncludes} \
               -I"$plugindir/include" \
               --plugin="protoc-gen-sol=$plugindir/plugin/gen_sol.py" \
               --sol_out=gen_runtime="ProtoBufRuntime.sol&solc_version=0.8.18:$2" \
                "$file"
            done
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

      packages.evm-contracts = mkEvmContracts {
        optimizerRuns = 10000;
        doCheck = true;
      };
    };
}

