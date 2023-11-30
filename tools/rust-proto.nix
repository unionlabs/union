{ ... }: {
  perSystem = { self', pkgs, proto, crane, system, config, ensureAtRepositoryRoot, ... }:
    let
      protoc-gen-tonic = crane.lib.buildPackage {
        pname = "protoc-gen-tonic";
        version = "0.0.1";
        src = pkgs.fetchFromGitHub {
          owner = "neoeinstein";
          repo = "protoc-gen-prost";
          rev = "1a6d3593622af18b75a4a79f545f8530cdaf444f";
          hash = "sha256-2DeFm3vE1q4KVMQlskJTHZnD7L5b6S0E50kZFF7ksN0=";
        };
      };

      cargo_toml = { name }: pkgs.runCommand "${name}-cargo_toml"
        { }
        ''
          cargotoml='${builtins.toJSON {
            package = {
              name = name;
              version = "0.0.0";
              edition = "2021";
            };
            lib = { doctest = false; };
            dependencies = {
              prost = { version = "0.11.0"; default-features = false; features = ["prost-derive"]; };
              ethers = { workspace = true; optional = true; features = ["rustls"]; };
              serde = { version = "1.0"; default-features = false; features = ["derive"]; optional = true; };
              tonic = { version = "0.9"; features = [ "gzip" ]; optional = true; };
              schemars = { version = "0.8.3"; default-features = false; optional = true; };
              serde-utils = { workspace = true; };
            };
            features = {
              default = [ "proto_full" "std" ];
              std = [ "prost/std" "serde/std" ];
              eth-abi = [ "ethers" "std" ];
              client = [ "tonic" ];
              json-schema = ["schemars"];
              # nix attrsets don't preserve order, use this to replace with the insertion point (see command below)
              PROTOC_INSERTION_POINT = 1;
            };
          }}'
          echo "cargo toml: $cargotoml"
          echo "$cargotoml" | ${pkgs.lib.meta.getExe pkgs.yj} -jt | sed 's/^PROTOC_INSERTION_POINT = 1$/## @@protoc_insertion_point(features)/' > $out
        '';

      all-protos-to-build = rec {
        wasmd = rec {
          src = "${proto.wasmd}/proto";
          proto-deps = [
            src
            google.src
            cosmos-sdk.src
            ics23.src
            "${proto.cosmosproto}/proto"
            "${proto.googleapis}"
          ];
        };
        cometbls = rec {
          src = "${proto.cometbls}/proto";
          proto-deps = [
            src
            google.src
            "${proto.gogoproto}"
          ];
        };
        uniond = rec {
          src = "${proto.uniond}";
          proto-deps = [
            src
          ];
        };
        galoisd = rec {
          src = "${proto.galoisd}";
          proto-deps = [
            src
          ];
        };
        google = rec {
          src = "${proto.gogoproto}/protobuf";
          proto-deps = [
            src
          ];
          additional-filter = "-path '*google/protobuf/*.proto'";
        };
        ibc-proto = rec {
          src = "${proto.ibcgo}/proto";
          proto-deps = [
            src
            google.src
            cosmos-sdk.src
            ics23.src
            "${proto.cosmosproto}/proto"
            "${proto.googleapis}"
          ];
        };
        ics23 = rec {
          src = "${proto.ics23}/proto";
          proto-deps = [
            src
          ];
        };
        cosmos-sdk = {
          src = "${proto.cosmossdk}/proto/cosmos";
          proto-deps = [
            "${proto.cosmossdk}/proto"
            google.src
            "${proto.cosmosproto}/proto"
            "${proto.googleapis}"
          ];
          # {msg,query}.proto have to be excluded since they only contain extensions, causing the generator to not create a file
          # SEE: https://github.com/neoeinstein/protoc-gen-prost/issues/61
          additional-filter = "-not -path '*cosmos/msg/v1/msg.proto' -not -path '*cosmos/query/v1/query.proto' -and -not -path '*/proto/tendermint/*'";
          fixup-script = ''
            sed -i 's/pub struct Validators/pub struct ValidatorsVec/' "./src/cosmos.staking.v1beta1.rs"
            sed -i 's/AllowList(Validators)/AllowList(ValidatorsVec)/' "./src/cosmos.staking.v1beta1.rs"
            sed -i 's/DenyList(Validators)/DenyList(ValidatorsVec)/' "./src/cosmos.staking.v1beta1.rs"
          '';
        };
      };

      fold-opts = attrs: with pkgs.lib; escapeShellArg (
        concatStringsSep "," (
          flatten (
            foldlAttrs
              (acc: opt-name: opt-value: acc ++ (
                # protoc splits on commas so we have to escape any in the attribute values
                foldlAttrs (acc: name: values: acc ++ (map (attr: "${opt-name}=${name}=${escape [","] attr}") values)) [ ] opt-value
              ))
              [ ]
              attrs
          )
        )
      );

      prost-opts =
        let
          # TODO(benluelo): structured rust attr builder?
          # something like:
          # { derive = ["Eq", "PartialOrd", "Ord"] }
          # { cfg_attr = [{ feature = "std"; } {serde = ["default"]} ]}
          attrs = {
            ord = ''#[derive(Eq, PartialOrd, Ord)]'';
            eq = ''#[derive(Eq)]'';

            # eth_abi = ''#[cfg_attr(feature = "ethers", derive(::ethers::contract::EthAbiType, ::ethers::contract::EthAbiCodec))]'';

            serde = ''#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]'';
            serde_default = ''#[cfg_attr(feature = "serde", serde(default))]'';
            serde_base64 = ''#[cfg_attr(feature = "serde", serde(with = "::serde_utils::base64"))]'';
            serde_inner_base64 = ''#[cfg_attr(feature = "serde", serde(with = "::serde_utils::inner_base64"))]'';

            jsonschema = ''#[cfg_attr(all(feature = "json-schema", feature = "std"), derive(::schemars::JsonSchema))]'';
            jsonschema_str = ''#[cfg_attr(all(feature = "json-schema", feature = "std"), schemars(with = "String"))]'';
          };
        in
        with attrs; {
          type_attribute = {
            # "." = [ eth_abi ];
            ".google.protobuf.Any" = [ serde eq ];
            ".google.protobuf.Timestamp" = [ serde ];
            ".google.protobuf.Duration" = [ serde eq ];
            ".ibc.core.client.v1" = [ serde ];
            ".ibc.core.client.v1.Height" = [ jsonschema ord ];
            ".ibc.core.commitment.v1" = [ serde ];
            ".ibc.core.commitment.v1.MerkleRoot" = [ jsonschema ];
            ".ibc.core.commitment.v1.MerklePrefix" = [ jsonschema ];
            ".ibc.core.channel.v1" = [ serde ];
            ".ibc.core.channel.v1.Channel" = [ jsonschema ];
            ".ibc.core.channel.v1.Counterparty" = [ jsonschema ];
            ".ibc.core.connection.v1" = [ serde ];
            ".ibc.core.connection.v1.ConnectionEnd" = [ jsonschema ];
            ".ibc.core.connection.v1.Counterparty" = [ jsonschema ];
            ".ibc.core.connection.v1.Version" = [ jsonschema ];
            ".ibc.core.types.v1" = [ serde ];
            ".ibc.applications.transfer.v1" = [ serde ];
            ".ibc.applications.transfer.v2" = [ serde ];
            ".ibc.applications.interchain_accounts.v1" = [ serde ];
            ".ibc.applications.interchain_accounts.controller.v1" = [ serde ];
            ".ibc.lightclients.wasm.v1" = [ serde ];
            ".ibc.lightclients.tendermint.v1.Fraction" = [ serde ];
            ".union.ibc.lightclients.ethereum.v1" = [ serde ];
            ".cosmos.ics23.v1" = [ serde ];
            ".cosmos.ics23.v1.LeafOp" = [ jsonschema eq ];
            ".cosmos.ics23.v1.InnerOp" = [ jsonschema eq ];
            ".cosmos.ics23.v1.ProofSpec" = [ eq jsonschema ];
            ".cosmos.ics23.v1.InnerSpec" = [ jsonschema eq ];
            ".cosmos.auth.v1beta1" = [ serde ];
            ".cosmos.upgrade.v1beta1" = [ serde ];
            ".cosmos.base.v1beta1" = [ serde ];
            ".cosmos.base.query.v1beta1" = [ serde ];
            ".cosmos.bank.v1beta1" = [ serde ];
          };

          field_attribute = {
            ".ibc.core.client.v1.Height" = [ serde_default ];
            ".ibc.core.commitment.v1.MerkleRoot.hash" = [ jsonschema_str serde_base64 ];
            ".ibc.core.commitment.v1.MerklePrefix.key_prefix" = [ jsonschema_str serde_base64 ];
            ".ibc.lightclients.wasm.v1.ClientState.data" = [ serde_base64 ];
            ".ibc.lightclients.wasm.v1.ClientState.code_id" = [ serde_base64 ];
            ".ibc.lightclients.wasm.v1.ConsensusState.data" = [ serde_base64 ];
            ".ibc.lightclients.wasm.v1.Header.data" = [ serde_base64 ];
            ".union.ibc.lightclients.ethereum.v1.SyncCommittee.aggregate_pubkey" = [ serde_base64 ];
            ".union.ibc.lightclients.ethereum.v1.SyncCommittee.pubkeys" = [ serde_inner_base64 ];
            ".union.ibc.lightclients.ethereum.v1.BeaconBlockHeader.parent_root" = [ serde_base64 ];
            ".union.ibc.lightclients.ethereum.v1.BeaconBlockHeader.state_root" = [ serde_base64 ];
            ".union.ibc.lightclients.ethereum.v1.BeaconBlockHeader.body_root" = [ serde_base64 ];
            ".union.ibc.lightclients.ethereum.v1.ExecutionPayloadHeader.parent_hash" = [ serde_base64 ];
            ".union.ibc.lightclients.ethereum.v1.ExecutionPayloadHeader.fee_recipient" = [ serde_base64 ];
            ".union.ibc.lightclients.ethereum.v1.ExecutionPayloadHeader.state_root" = [ serde_base64 ];
            ".union.ibc.lightclients.ethereum.v1.ExecutionPayloadHeader.receipts_root" = [ serde_base64 ];
            ".union.ibc.lightclients.ethereum.v1.ExecutionPayloadHeader.logs_bloom" = [ serde_base64 ];
            ".union.ibc.lightclients.ethereum.v1.ExecutionPayloadHeader.prev_randao" = [ serde_base64 ];
            ".union.ibc.lightclients.ethereum.v1.ExecutionPayloadHeader.extra_data" = [ serde_base64 ];
            ".union.ibc.lightclients.ethereum.v1.ExecutionPayloadHeader.base_fee_per_gas" = [ serde_base64 ];
            ".union.ibc.lightclients.ethereum.v1.ExecutionPayloadHeader.block_hash" = [ serde_base64 ];
            ".union.ibc.lightclients.ethereum.v1.ExecutionPayloadHeader.transactions_root" = [ serde_base64 ];
            ".union.ibc.lightclients.ethereum.v1.ExecutionPayloadHeader.withdrawals_root" = [ serde_base64 ];
            ".union.ibc.lightclients.ethereum.v1.LightClientHeader.execution_branch" = [ serde_inner_base64 ];
            ".union.ibc.lightclients.ethereum.v1.LightClientUpdate.next_sync_committee_branch" = [ serde_inner_base64 ];
            ".union.ibc.lightclients.ethereum.v1.LightClientUpdate.finality_branch" = [ serde_inner_base64 ];
            ".union.ibc.lightclients.ethereum.v1.SyncAggregate.sync_committee_bits" = [ serde_base64 ];
            ".union.ibc.lightclients.ethereum.v1.SyncAggregate.sync_committee_signature" = [ serde_base64 ];
            ".union.ibc.lightclients.ethereum.v1.Proof.key" = [ serde_base64 ];
            ".union.ibc.lightclients.ethereum.v1.Proof.value" = [ serde_base64 ];
            ".union.ibc.lightclients.ethereum.v1.Proof.proof" = [ serde_inner_base64 ];
            ".cosmos.ics23.v1.LeafOp.prehash_key" = [ serde_default ];
            ".cosmos.ics23.v1.LeafOp.prefix" = [ jsonschema_str serde_base64 ];
            ".cosmos.ics23.v1.InnerOp.prefix" = [ serde_base64 jsonschema_str ];
            ".cosmos.ics23.v1.InnerOp.suffix" = [ serde_base64 jsonschema_str ];
            ".cosmos.ics23.v1.ProofSpec.max_depth" = [ serde_default ];
            ".cosmos.ics23.v1.ProofSpec.min_depth" = [ serde_default ];
            ".cosmos.ics23.v1.InnerSpec.empty_child" = [ serde_default jsonschema_str serde_base64 ];
          };
        };

      tonic-opts = {
        client_mod_attribute = { "." = [ ''#[cfg(feature = "client")]'' ]; };
        # server_mod_attribute = { "." = [ ''#[cfg(feature = "server")]'' ]; };
      };

      proto-inputs = name: { src, additional-filter ? null, ... }:
        let
          af = if additional-filter != null then "-and " + additional-filter else "";
        in
        pkgs.runCommand
          "${name}-inputs"
          { }
          ''
            find ${src} -type f -name '*.proto' ${af} > $out
          '';

      fixup-scripts = with pkgs.lib; concatStringsSep "\n\n" (
        flatten (
          mapAttrsToList
            (
              name: { fixup-script ? null, ... }:
                optionalString
                  (fixup-script != null)
                  ''
                    echo "[FIXUP] ${name}"

                    ${fixup-script}
                  ''
            )
            all-protos-to-build
        )
      );

      includes = with pkgs.lib; concatStringsSep " " (
        flatten (
          mapAttrsToList
            (
              _: { proto-deps, ... }:
                map (src: ''-I"${src}"'') proto-deps
            )
            all-protos-to-build
        ));

      rust-proto = pkgs.stdenv.mkDerivation {
        name = "rust-proto";
        pname = "rust-proto";
        src = pkgs.linkFarm "rust-proto-srcs" (pkgs.lib.mapAttrsToList (name: { src, ... }: { name = name + "-protos"; path = src; }) all-protos-to-build);
        buildInputs = [ pkgs.protobuf protoc-gen-tonic config.treefmt.build.programs.rustfmt pkgs.taplo ] ++ (if pkgs.stdenv.isDarwin then [ pkgs.libiconv ] else [ ]);
        buildPhase = ''
          mkdir $out

          readarray -t protos < <(cat ${pkgs.lib.concatStringsSep " " (pkgs.lib.mapAttrsToList proto-inputs all-protos-to-build)})

          for file in "''${protos[@]}"; do
            echo "[FOUND] $file"
          done

          protoc "''${protos[@]}" \
            --prost_out=./src \
            --prost_opt=compile_well_known_types=true,${fold-opts prost-opts} \
            --tonic_out=./src \
            --tonic_opt=compile_well_known_types=true,no_server=true,${fold-opts tonic-opts} \
            --prost-crate_out=. \
            --prost-crate_opt=package_separator="+",gen_crate=${cargo_toml { name = "protos"; }} \
            ${includes}

          ${fixup-scripts}

          # prepend clippy allow to root lib.rs file
          echo -e "#![allow(clippy::all)]\n$(cat ./src/lib.rs)" > ./src/lib.rs

          # format generated files
          # echo 'formatter = { rust = { options = []}}' > treefmt.toml
          # touch flake.nix # treefmt looks for this file to find the project root
          # treefmt -C . --no-cache --config-file="treefmt.toml"

          # normalize comments in generated code
          for i in $(find . -name "*.rs" -type f); do
            echo "[FORMAT] $i"
            rustfmt --config normalize_comments=true --edition "2021" "$i"
          done

          taplo format ./Cargo.toml

          cp -r ./src $out/
          cp -r ./Cargo.toml $out/
        '';
      };
    in
    {
      packages.rust-proto = rust-proto;

      packages.generate-rust-proto = pkgs.writeShellApplication {
        name = "generate-rust-proto";
        runtimeInputs = [ rust-proto pkgs.rsync ];
        text = ''
          ${ensureAtRepositoryRoot}

          outdir="generated/rust/"

          mkdir -p "$outdir"

          rsync -rL --chmod=ugo=rwX --delete ${rust-proto}/ $outdir

          echo "Generation successful!"
        '';
      };
    };
}
