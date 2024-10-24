_: {
  perSystem =
    {
      self',
      pkgs,
      proto,
      crane,
      system,
      config,
      ensureAtRepositoryRoot,
      mkCi,
      ...
    }:
    let
      protoc-gen-tonic = crane.lib.buildPackage {
        pname = "protoc-gen-tonic";
        version = "0.0.1";
        doCheck = false;
        src = builtins.fetchGit {
          url = "https://github.com/neoeinstein/protoc-gen-prost";
          rev = "0548ae244f5780cdf0790ebf48b497a5df1acfc4";
        };
      };

      cargo_toml =
        { name }:
        let
          toml = {
            package = {
              inherit name;
              version = "0.0.0";
              edition = "2021";
            };
            lib = {
              doctest = false;
            };
            dependencies = {
              prost = {
                workspace = true;
                features = [ "prost-derive" ];
              };
              ethers = {
                workspace = true;
                optional = true;
                features = [ "rustls" ];
              };
              serde = {
                workspace = true;
                features = [ "derive" ];
                optional = true;
              };
              tonic = {
                workspace = true;
                features = [
                  "codegen"
                  "prost"
                  "gzip"
                  "transport"
                ];
                optional = true;
              };
              schemars = {
                workspace = true;
                optional = true;
              };
              serde-utils = {
                workspace = true;
              };
              chrono = {
                workspace = true;
                features = [ "alloc" ];
              };
              # https://github.com/influxdata/pbjson/pull/118
              pbjson-types = {
                git = "https://github.com/recoord/pbjson";
                rev = "2b7a8e4c2c83a40d04beed46aa26ab97a39a81fe";
              };
            };
            features = {
              default = [
                "proto_full"
                "std"
              ];
              std = [
                "prost/std"
                "serde/std"
              ];
              eth-abi = [
                "ethers"
                "std"
              ];
              client = [ "tonic" ];
              json-schema = [ "schemars" ];
              # nix attrsets don't preserve order, use this to replace with the insertion point (see command below)
              PROTOC_INSERTION_POINT = 1;
            };
            lints = {
              workspace = true;
            };
          };
        in
        pkgs.runCommand "${name}-cargo_toml" { } ''
          cargotoml='${builtins.toJSON toml}'
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
          # inject https://github.com/cometbft/cometbft/blob/main/proto/cometbft/crypto/v1/keys.proto#L17
          # note that this may cause issues if we decode proto bytes from another chain with this key type, since we use field identifier 3 for bn254 signatures.
          # and yes prost is incredibly cursed. we have to annotate the field that uses the generated oneof `Sum` type with the additional enum tag for some reason? no clue why this information has to be duplicated.
          # ask me how much time i wasted figuring this out
          fixup-script = ''
            # sed -i 's/#\[prost(oneof = "public_key::Sum", tags = "1, 2, 3")\]/#\[prost(oneof = "public_key::Sum", tags = "1, 2, 3, 4")\]/' "./src/cometbft.crypto.v1.rs"
            # sed -i 's/Bn254(::prost::alloc::vec::Vec<u8>),/Bn254(::prost::alloc::vec::Vec<u8>),#\[prost(bytes, tag = "4")\]Bls12381(::prost::alloc::vec::Vec<u8>),/' "./src/cometbft.crypto.rs"

            # required until https://github.com/tokio-rs/prost/issues/507 is fixed
            sed -i 's/pub sum: ::core::option::Option<public_key::Sum>,/#\[cfg_attr(feature = "serde", serde(flatten))\]pub sum: ::core::option::Option<public_key::Sum>,/' "./src/cometbft.crypto.v1.rs"
            sed -i 's/pub enum Sum {/#\[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))\]pub enum Sum {/' "./src/cometbft.crypto.v1.rs"

            # i can't figure out how to add attributes to the variants directly, possibly related to the issue linked above
            sed -i 's/Ed25519(::prost::alloc::vec::Vec<u8>)/#\[serde(rename = "tendermint\/PubKeyEd25519")\]Ed25519(#[serde(with = "::serde_utils::base64")] ::prost::alloc::vec::Vec<u8>)/' "./src/cometbft.crypto.v1.rs"
            sed -i 's/Secp256k1(::prost::alloc::vec::Vec<u8>)/#\[serde(rename = "tendermint\/PubKeySecp256k1")\]Secp256k1(#[serde(with = "::serde_utils::base64")] ::prost::alloc::vec::Vec<u8>)/' "./src/cometbft.crypto.v1.rs"
            sed -i 's/Bn254(::prost::alloc::vec::Vec<u8>)/#\[serde(rename = "cometbft\/PubKeyBn254")\]Bn254(#[serde(with = "::serde_utils::base64")] ::prost::alloc::vec::Vec<u8>)/' "./src/cometbft.crypto.v1.rs"
            sed -i 's/Bls12381(::prost::alloc::vec::Vec<u8>)/#\[serde(rename = "cometbft\/PubKeyBls12381")\]Bls12381(#[serde(with = "::serde_utils::base64")] ::prost::alloc::vec::Vec<u8>)/' "./src/cometbft.crypto.v1.rs"



            # required until https://github.com/tokio-rs/prost/issues/507 is fixed
            sed -i 's/pub sum: ::core::option::Option<evidence::Sum>,/#\[cfg_attr(feature = "serde", serde(flatten))\]pub sum: ::core::option::Option<evidence::Sum>,/' "./src/cometbft.types.v1.rs"
            sed -i 's/pub enum Sum {/#\[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))\]pub enum Sum {/' "./src/cometbft.types.v1.rs"

            sed -i 's/DuplicateVoteEvidence(/#\[serde(rename = "tendermint\/DuplicateVoteEvidence")\]DuplicateVoteEvidence(/' "./src/cometbft.types.v1.rs"
            sed -i 's/LightClientAttackEvidence(/#\[serde(rename = "tendermint\/LightClientAttackEvidence")\]LightClientAttackEvidence(/' "./src/cometbft.types.v1.rs"
          '';
        };

        uniond = rec {
          src = "${proto.uniond}";
          proto-deps = [
            src
            "${proto.cosmossdk}/x/bank/proto"
            "${proto.cosmossdk}/x/staking/proto"
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
          fixup-script = ''
            echo "pub use pbjson_types::*;" >> "./src/google.protobuf.rs"
          '';
        };
        ibc-proto = rec {
          src = "${proto.ibc-go}/proto";
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
          additional-filter = "-not -path '*cosmos/msg/textual/v1/textual.proto' -not -path '*cosmos/msg/v1/msg.proto' -not -path '*cosmos/query/v1/query.proto' -and -not -path '*/proto/tendermint/*'";
          fixup-script = ''
            sed -i 's/pub struct Validators/pub struct ValidatorsList/' "./src/cosmos.staking.v1beta1.rs"
            sed -i 's/impl ::prost::Name for Validators/impl ::prost::Name for ValidatorsList/' "./src/cosmos.staking.v1beta1.rs"
            sed -i 's/AllowList(Validators)/AllowList(ValidatorsList)/' "./src/cosmos.staking.v1beta1.rs"
            sed -i 's/DenyList(Validators)/DenyList(ValidatorsList)/' "./src/cosmos.staking.v1beta1.rs"
          '';
        };
        cosmos-sdk-bank = {
          src = "${proto.cosmossdk}/x/bank/proto";
          proto-deps = [ ];
        };
        cosmos-sdk-staking = {
          src = "${proto.cosmossdk}/x/staking/proto";
          proto-deps = [ ];
        };
      };

      fold-opts =
        attrs:
        with pkgs.lib;
        escapeShellArg (
          concatStringsSep "," (
            flatten (
              foldlAttrs (
                acc: opt-name: opt-value:
                acc
                ++ (
                  # protoc splits on commas so we have to escape any in the attribute values
                  foldlAttrs (
                    acc: name: values:
                    acc ++ (map (attr: "${opt-name}=${name}=${escape [ "," ] attr}") values)
                  ) [ ] opt-value
                )
              ) [ ] attrs
            )
          )
        );

      prost-opts =
        let
          # TODO(benluelo): structured rust attr builder?
          # something like:
          # { derive = ["Eq", "PartialOrd", "Ord"] }
          # { cfg_attr = [{ feature = "std"; } {serde = ["default"]} ]}
          # ord = ''#[derive(Eq, PartialOrd, Ord)]'';
          # eq = ''#[derive(Eq)]'';

          # eth_abi = ''#[cfg_attr(feature = "ethers", derive(::ethers::contract::EthAbiType, ::ethers::contract::EthAbiCodec))]'';

          serde = ''#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]'';
          serde_default = ''#[cfg_attr(feature = "serde", serde(default))]'';
          # serde_flatten = ''#[cfg_attr(feature = "serde", serde(flatten))]'';
          serde_string = ''#[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]'';
          serde_base64 = ''#[cfg_attr(feature = "serde", serde(with = "::serde_utils::base64"))]'';
          # serde_base64_opt = ''#[cfg_attr(feature = "serde", serde(with = "::serde_utils::base64_opt"))]'';
          serde_base64_opt_default = ''#[cfg_attr(feature = "serde", serde(with = "::serde_utils::base64_opt_default"))]'';
          serde_inner_base64 = ''#[cfg_attr(feature = "serde", serde(with = "::serde_utils::inner_base64"))]'';
          serde_hex_upper_unprefixed = ''#[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_upper_unprefixed"))]'';

          # jsonschema = ''#[cfg_attr(all(feature = "json-schema", feature = "std"), derive(::schemars::JsonSchema))]'';
          # jsonschema_str = ''#[cfg_attr(all(feature = "json-schema", feature = "std"), schemars(with = "String"))]'';
          serde_alias = alias: ''#[serde(alias = "${alias}")]'';
        in
        {
          type_attribute = {
            ".google.protobuf.Any" = [ serde ];
            ".google.protobuf.Duration" = [ serde ];
            ".google.protobuf.Timestamp" = [ serde ];

            ".ibc.core.client.v1" = [ serde ];
            ".ibc.core.client.v1.Height" = [ ];

            ".ibc.core.commitment.v1" = [ serde ];
            ".ibc.core.commitment.v1.MerklePrefix" = [ ];
            ".ibc.core.commitment.v1.MerkleRoot" = [ ];

            ".ibc.core.channel.v1" = [ serde ];
            ".ibc.core.channel.v1.Channel" = [ ];
            ".ibc.core.channel.v1.Counterparty" = [ ];

            ".ibc.core.connection.v1" = [ serde ];
            ".ibc.core.connection.v1.ConnectionEnd" = [ ];
            ".ibc.core.connection.v1.Counterparty" = [ ];
            ".ibc.core.connection.v1.Version" = [ ];

            ".ibc.core.types.v1" = [ serde ];

            ".ibc.applications.interchain_accounts.controller.v1" = [ serde ];
            ".ibc.applications.interchain_accounts.v1" = [ serde ];

            ".ibc.applications.transfer.v1" = [ serde ];
            ".ibc.applications.transfer.v2" = [ serde ];

            ".ibc.lightclients.wasm.v1" = [ serde ];

            ".ibc.lightclients.tendermint.v1.Fraction" = [ serde ];

            ".union.ibc.lightclients.ethereum.v1" = [ serde ];

            ".cosmos.ics23.v1" = [ serde ];
            ".cosmos.ics23.v1.InnerOp" = [ ];
            ".cosmos.ics23.v1.InnerSpec" = [ ];
            ".cosmos.ics23.v1.LeafOp" = [ ];
            ".cosmos.ics23.v1.ProofSpec" = [ ];

            ".cosmos.auth.v1beta1" = [ serde ];

            ".cosmos.upgrade.v1beta1" = [ serde ];

            ".cosmos.base.v1beta1" = [ serde ];
            ".cosmos.base.query.v1beta1" = [ serde ];

            ".cosmos.bank.v1beta1" = [ serde ];

            ".cometbft.types.v1.Block" = [ serde ];
            ".cometbft.types.v1.BlockID" = [ serde ];
            ".cometbft.types.v1.Commit" = [ serde ];
            ".cometbft.types.v1.CommitSig" = [ serde ];
            ".cometbft.types.v1.Data" = [ serde ];
            ".cometbft.types.v1.DuplicateVoteEvidence" = [ serde ];
            ".cometbft.types.v1.Evidence" = [ serde ];
            ".cometbft.types.v1.EvidenceList" = [ serde ];
            ".cometbft.types.v1.Header" = [ serde ];
            ".cometbft.types.v1.LightBlock" = [ serde ];
            ".cometbft.types.v1.LightClientAttackEvidence" = [ serde ];
            ".cometbft.types.v1.PartSetHeader" = [ serde ];
            ".cometbft.types.v1.SignedHeader" = [ serde ];
            ".cometbft.types.v1.TxProof" = [ serde ];
            ".cometbft.types.v1.Validator" = [ serde ];
            ".cometbft.types.v1.ValidatorSet" = [ serde ];
            ".cometbft.types.v1.Vote" = [ serde ];

            ".cometbft.version.v1.Consensus" = [ serde ];

            ".cometbft.abci.v1.ExecTxResult" = [ serde ];
            ".cometbft.abci.v1.Event" = [ serde ];
            ".cometbft.abci.v1.EventAttribute" = [ serde ];
            ".cometbft.abci.v1.QueryResponse" = [ serde ];

            ".cometbft.crypto.v1.PublicKey" = [ serde ];
            # ".cometbft.crypto.v1.PublicKey.sum" = [ serde ];
            ".cometbft.crypto.v1.ProofOps" = [ serde ];
            ".cometbft.crypto.v1.ProofOp" = [ serde ];
            ".cometbft.crypto.v1.Proof" = [ serde ];

            ".cometbft.p2p.v1.DefaultNodeInfo" = [ serde ];
            ".cometbft.p2p.v1.DefaultNodeInfoOther" = [ serde ];
            ".cometbft.p2p.v1.ProtocolVersion" = [ serde ];

            # ".cometbft.types.v1.Validator" = [ serde ];
          };

          field_attribute = {
            ".ibc.core.client.v1.Height" = [ serde_default ];

            ".ibc.core.commitment.v1.MerkleRoot.hash" = [ serde_base64 ];

            ".ibc.core.commitment.v1.MerklePrefix.key_prefix" = [ serde_base64 ];

            ".ibc.lightclients.wasm.v1.ClientState.data" = [ serde_base64 ];
            ".ibc.lightclients.wasm.v1.ClientState.checksum" = [ serde_base64 ];

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

            ".union.ibc.lightclients.ethereum.v1.LightClientUpdate.next_sync_committee_branch" = [
              serde_inner_base64
            ];
            ".union.ibc.lightclients.ethereum.v1.LightClientUpdate.finality_branch" = [ serde_inner_base64 ];

            ".union.ibc.lightclients.ethereum.v1.SyncAggregate.sync_committee_bits" = [ serde_base64 ];
            ".union.ibc.lightclients.ethereum.v1.SyncAggregate.sync_committee_signature" = [ serde_base64 ];

            ".union.ibc.lightclients.ethereum.v1.Proof.key" = [ serde_base64 ];
            ".union.ibc.lightclients.ethereum.v1.Proof.value" = [ serde_base64 ];
            ".union.ibc.lightclients.ethereum.v1.Proof.proof" = [ serde_inner_base64 ];

            ".cosmos.ics23.v1.LeafOp.hash" = [ serde_default ];
            ".cosmos.ics23.v1.LeafOp.prehash_key" = [ serde_default ];
            ".cosmos.ics23.v1.LeafOp.prehash_value" = [ serde_default ];
            ".cosmos.ics23.v1.LeafOp.length" = [ serde_default ];
            ".cosmos.ics23.v1.LeafOp.prefix" = [
              serde_default
              serde_base64
            ];

            ".cosmos.ics23.v1.InnerOp.prefix" = [
              serde_base64
              serde_default
            ];
            ".cosmos.ics23.v1.InnerOp.suffix" = [
              serde_base64
              serde_default
            ];

            ".cosmos.ics23.v1.ProofSpec.max_depth" = [ serde_default ];
            ".cosmos.ics23.v1.ProofSpec.min_depth" = [ serde_default ];
            ".cosmos.ics23.v1.ProofSpec.prehash_key_before_comparison" = [ serde_default ];

            ".cosmos.ics23.v1.InnerSpec.empty_child" = [
              serde_default
              serde_base64
            ];

            ".cosmos.ics23.v1.ExistenceProof.key" = [ serde_base64 ];
            ".cosmos.ics23.v1.ExistenceProof.value" = [ serde_base64 ];
            ".cosmos.ics23.v1.ExistenceProof.path" = [ serde_default ];
            ".cosmos.ics23.v1.ExistenceProof.leaf" = [ serde_default ];

            ".cosmos.ics23.v1.NonExistenceProof.value" = [ serde_base64 ];

            ".cometbft.types.v1.Header.last_commit_hash" = [ serde_hex_upper_unprefixed ];
            ".cometbft.types.v1.Header.data_hash" = [ serde_hex_upper_unprefixed ];
            ".cometbft.types.v1.Header.validators_hash" = [ serde_hex_upper_unprefixed ];
            ".cometbft.types.v1.Header.next_validators_hash" = [ serde_hex_upper_unprefixed ];
            ".cometbft.types.v1.Header.consensus_hash" = [ serde_hex_upper_unprefixed ];
            ".cometbft.types.v1.Header.app_hash" = [ serde_hex_upper_unprefixed ];
            ".cometbft.types.v1.Header.last_results_hash" = [ serde_hex_upper_unprefixed ];
            ".cometbft.types.v1.Header.evidence_hash" = [ serde_hex_upper_unprefixed ];
            ".cometbft.types.v1.Header.proposer_address" = [ serde_hex_upper_unprefixed ];
            ".cometbft.types.v1.Header.height" = [ serde_string ];

            # this type is so cursed
            ".cometbft.types.v1.BlockID.hash" = [ serde_hex_upper_unprefixed ];
            ".cometbft.types.v1.BlockID.part_set_header" = [ (serde_alias "parts") ];

            ".cometbft.types.v1.PartSetHeader.hash" = [ serde_hex_upper_unprefixed ];
            ".cometbft.types.v1.Commit.height" = [ serde_string ];
            ".cometbft.types.v1.CommitSig.signature" = [ serde_base64_opt_default ];
            ".cometbft.types.v1.CommitSig.validator_address" = [ serde_hex_upper_unprefixed ];
            ".cometbft.types.v1.CommitSig.timestamp" = [
              ''
                #[cfg_attr(
                    feature = "serde",
                    serde(with = "::serde_utils::parse_from_rfc3339_string_but_0001_01_01T00_00_00Z_is_none")
                )]''
            ];

            ".cometbft.version.v1.Consensus.block" = [ serde_string ];
            ".cometbft.version.v1.Consensus.app" = [ serde_default ];

            ".cometbft.abci.v1.QueryResponse.index" = [ serde_string ];
            ".cometbft.abci.v1.QueryResponse.height" = [ serde_string ];
            ".cometbft.abci.v1.QueryResponse.key" = [ serde_base64_opt_default ];
            ".cometbft.abci.v1.QueryResponse.value" = [ serde_base64_opt_default ];
            ".cometbft.abci.v1.QueryResponse.proof_ops" = [ (serde_alias "proofOps") ];

            ".cometbft.crypto.v1.ProofOp.key" = [ serde_base64 ];
            ".cometbft.crypto.v1.ProofOp.data" = [ serde_base64 ];

            ".cometbft.crypto.v1.Proof.total" = [ serde_string ];
            ".cometbft.crypto.v1.Proof.index" = [ serde_string ];
            ".cometbft.crypto.v1.Proof.leaf_hash" = [ serde_base64 ];
            ".cometbft.crypto.v1.Proof.aunts" = [ serde_inner_base64 ];

            ".cometbft.p2p.v1.DefaultNodeInfo.channels" = [ serde_hex_upper_unprefixed ];
            ".cometbft.p2p.v1.DefaultNodeInfo.default_node_id" = [ (serde_alias "id") ];

            ".cometbft.p2p.v1.ProtocolVersion.p2p" = [ serde_string ];
            ".cometbft.p2p.v1.ProtocolVersion.block" = [ serde_string ];
            ".cometbft.p2p.v1.ProtocolVersion.app" = [ serde_string ];

            ".cometbft.types.v1.Validator.address" = [ serde_hex_upper_unprefixed ];
            ".cometbft.types.v1.Validator.voting_power" = [ serde_string ];
            ".cometbft.types.v1.Validator.proposer_priority" = [ serde_string ];
            ".cometbft.types.v1.Validator.pub_key_type" = [ serde_default ];
            ".cometbft.types.v1.Validator.pub_key_bytes" = [ serde_default ];

            ".cometbft.types.v1.Data.txs" = [ serde_inner_base64 ];

            ".cometbft.types.v1.Vote.height" = [ serde_string ];
            ".cometbft.types.v1.Vote.validator_address" = [ serde_hex_upper_unprefixed ];
            ".cometbft.types.v1.Vote.signature" = [ serde_base64 ];
            ".cometbft.types.v1.Vote.extension" = [ serde_base64_opt_default ];
            ".cometbft.types.v1.Vote.extension_signature" = [ serde_base64_opt_default ];

            ".cometbft.types.v1.TxProof.root_hash" = [ serde_hex_upper_unprefixed ];
            ".cometbft.types.v1.TxProof.data" = [ serde_base64 ];

            ".cometbft.abci.v1.ExecTxResult.data" = [ serde_base64_opt_default ];
            ".cometbft.abci.v1.ExecTxResult.gas_wanted" = [ serde_string ];
            ".cometbft.abci.v1.ExecTxResult.gas_used" = [ serde_string ];

            # ".cometbft.types.v1.Vote.timestamp" = [
            #   ''#[cfg_attr(
            #       feature = "serde",
            #       serde(with = "::serde_utils::parse_from_rfc3339_string_but_0001_01_01T00_00_00Z_is_none")
            #   )]''
            # ];

            ".cometbft.types.v1.DuplicateVoteEvidence.total_voting_power" = [
              (serde_alias "TotalVotingPower")
              serde_string
            ];
            ".cometbft.types.v1.DuplicateVoteEvidence.validator_power" = [
              (serde_alias "ValidatorPower")
              serde_string
            ];
            ".cometbft.types.v1.DuplicateVoteEvidence.timestamp" = [ (serde_alias "Timestamp") ];

            ".cometbft.types.v1.LightClientAttackEvidence.common_height" = [ serde_string ];
            ".cometbft.types.v1.LightClientAttackEvidence.total_voting_power" = [ serde_string ];
            # ".cometbft.crypto.v1.PublicKey.sum" = [ serde_flatten ];
          };

          enum_attribute = {
            # ".cometbft.crypto.v1.PublicKey.sum.Ed25519" = [ (serde_alias "tendermint/PubKeyEd25519") ];
            # ".cometbft.types.v1.Evidence.sum.DuplicateVoteEvidence" = [ (serde_alias "tendermint/DuplicateVoteEvidence") ];
            # ".cometbft.types.v1.Evidence.sum.LightClientAttackEvidence" = [ (serde_alias "tendermint/LightClientAttackEvidence") ];
          };
        };

      tonic-opts = {
        client_mod_attribute = {
          "." = [ ''#[cfg(feature = "client")]'' ];
        };
        # server_mod_attribute = { "." = [ ''#[cfg(feature = "server")]'' ]; };
      };

      proto-inputs =
        name:
        {
          src,
          additional-filter ? null,
          ...
        }:
        let
          af = if additional-filter != null then "-and " + additional-filter else "";
        in
        pkgs.runCommand "${name}-inputs" { } ''
          find ${src} -type f -name '*.proto' ${af} > $out
        '';

      fixup-scripts =
        with pkgs.lib;
        concatStringsSep "\n\n" (
          flatten (
            mapAttrsToList (
              name:
              {
                fixup-script ? null,
                ...
              }:
              optionalString (fixup-script != null) ''
                echo "[FIXUP] ${name}"

                ${fixup-script}
              ''
            ) all-protos-to-build
          )
        );

      includes =
        with pkgs.lib;
        concatStringsSep " " (
          flatten (
            mapAttrsToList (_: { proto-deps, ... }: map (src: ''-I"${src}"'') proto-deps) all-protos-to-build
          )
        );

      rust-proto = pkgs.stdenv.mkDerivation {
        name = "rust-proto";
        pname = "rust-proto";
        src = pkgs.linkFarm "rust-proto-srcs" (
          pkgs.lib.mapAttrsToList (
            name:
            { src, ... }:
            {
              name = name + "-protos";
              path = src;
            }
          ) all-protos-to-build
        );
        buildInputs = [
          pkgs.protobuf
          protoc-gen-tonic
          config.treefmt.build.programs.rustfmt
          pkgs.taplo
        ] ++ (if pkgs.stdenv.isDarwin then [ pkgs.libiconv ] else [ ]);
        buildPhase = ''
          mkdir $out

          readarray -t protos < <(cat ${pkgs.lib.concatStringsSep " " (pkgs.lib.mapAttrsToList proto-inputs all-protos-to-build)})

          for file in "''${protos[@]}"; do
            echo "[FOUND] $file"
          done

          protoc "''${protos[@]}" \
            --prost_opt=compile_well_known_types \
            --prost_opt=extern_path=.google.protobuf=::pbjson_types \
            --prost_out=./src \
            --prost_opt=enable_type_names=true,compile_well_known_types=true,${fold-opts prost-opts} \
            --tonic_out=./src \
            --tonic_opt=compile_well_known_types=true,no_server=true,${fold-opts tonic-opts} \
            --prost-crate_out=. \
            --prost-crate_opt=package_separator="+",gen_crate=${cargo_toml { name = "protos"; }} \
            ${includes}

          ${fixup-scripts}

          # prepend clippy and rustdoc allow to root lib.rs file
          echo -e "#![allow(clippy::all, rustdoc::all)]\n$(cat ./src/lib.rs)" > ./src/lib.rs

          # format generated files
          # echo 'formatter = { rust = { options = []}}' > treefmt.toml
          # touch flake.nix # treefmt looks for this file to find the project root
          # treefmt -C . --no-cache --config-file="treefmt.toml"

          # normalize comments in generated code
          for i in $(find . -name "*.rs" -type f); do
            echo "[FORMAT] $i"
            rustfmt --config-path=${../rustfmt.toml} --config normalize_comments=true --edition "2021" "$i"
          done

          taplo format --config=${../taplo.toml} ./Cargo.toml

          cp -r ./src $out/
          cp -r ./Cargo.toml $out/
        '';
      };
    in
    {
      packages.rust-proto = mkCi false rust-proto;

      packages.generate-rust-proto = mkCi false (
        pkgs.writeShellApplication {
          name = "generate-rust-proto";
          runtimeInputs = [
            rust-proto
            pkgs.rsync
          ];
          text = ''
            ${ensureAtRepositoryRoot}

            outdir="generated/rust/protos/"

            mkdir -p "$outdir"

            rsync -rL --chmod=ugo=rwX --delete ${rust-proto}/ $outdir

            echo "Generation successful!"
          '';
        }
      );

      checks = {
        rust-proto-check = mkCi false (
          pkgs.stdenv.mkDerivation {
            name = "rust-proto-is-committed";
            description = "check that rust protos in git repo are the same as those that are generated in rust-proto derivation";
            src = ../.;
            buildInputs = [ pkgs.git ];
            doCheck = true;
            checkPhase = ''
              rust_protos_in_git_repo=./generated/rust/protos
              rust_protos_in_derivation=${self'.packages.rust-proto}
              git --no-pager diff --exit-code --no-index $rust_protos_in_git_repo $rust_protos_in_derivation
              touch $out
            '';
          }
        );
      };
    };
}
