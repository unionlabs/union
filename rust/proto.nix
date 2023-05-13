{ ... }: {
  perSystem = { self', pkgs, proto, crane, system, ... }: {
    packages.generate-rust-proto = pkgs.writeShellApplication {
      name = "generate-rust-proto";
      runtimeInputs = [ pkgs.protobuf ];
      text =
        let
          protoc-gen-tonic = crane.lib.buildPackage {
            pname = "protoc-gen-tonic";
            version = "0.0.1";
            src = pkgs.fetchFromGitHub {
              owner = "neoeinstein";
              repo = "protoc-gen-prost";
              rev = "44d3fc389cd05e4e8060fe296770677d1f066e16";
              hash = "sha256-NqZdLHRo3tIudGiU9hev4oX8jsoz1aANxw9scBMGCqk=";
            };
          };

          cargo_toml = { name, dependencies }: pkgs.runCommand "${name}-cargo_toml"
            { }
            ''
              cargotoml='${builtins.toJSON {
                package = {
                  name = name;
                  version = "0.1.0";
                  edition = "2021";
                };
                dependencies = {
                  prost = "0.11.0";
                  prost-types = "*";
                  pbjson-types = "0.5";
                  pbjson = "*";
                  serde = "1.0";
                  tonic = { version = "0.8"; features = [ "gzip" ]; };
                } // dependencies;
                features = {
                  default = [ "proto_full" ];
                  # nix attrsets don't preserve order, use this to replace with the insertion point (see command below)
                  PROTOC_INSERTION_POINT = 1;
                };
              }}'
              echo "cargo toml: $cargotoml"
              echo "$cargotoml" | ${pkgs.lib.meta.getExe pkgs.yj} -jt | sed 's/^PROTOC_INSERTION_POINT = 1$/## @@protoc_insertion_point(features)/' > $out
              # echo -e "\n" >> $out
            '';

          attrs = {
            ord = ''#[derive(Eq, PartialOrd, Ord)]'';
            eq = ''#[derive(Eq)]'';

            serde = ''#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]'';
            serde_default = ''#[cfg_attr(feature = "std", serde(default))]'';
            serde_base64 = ''#[cfg_attr(feature = "std", serde(with = "crate::base64"))]'';
            serde_inner_base64 = ''#[serde(with = "crate::inner_base64")]'';

            jsonschema = ''#[cfg_attr(all(feature = "json-schema", feature = "std"), derive(::schemars::JsonSchema))]'';
            jsonschema_str = ''#[cfg_attr(all(feature = "json-schema", feature = "std"), schemars(with = "String"))]'';
          };

          all-proto-srcs = with proto; [
            "${cometbls}/proto"
            # "${cosmossdk}/proto"
            # "${ibcgo}/proto"
            # "${cosmosproto}/proto"
            # "${ics23}/proto"
            # "${googleapis}"
            # "${gogoproto}"
            # "${unionpd}"
            # "${uniond}"
          ];

          # TODO(benluelo): refactor into an attrset with name as the key
          all-proto-build = with proto; [
            {
              name = "cometbls";
              src = "${cometbls}/proto";
              proto-deps = [
                "${cometbls}/proto"
                "${gogoproto}"
              ];
            }
            {
              name = "ibc-go";
              src = "${ibcgo}/proto";
              proto-deps = [
                "${ibcgo}/proto"
                "${gogoproto}"
                "${cosmossdk}/proto"
                "${cosmosproto}/proto"
                "${ics23}/proto"
                "${googleapis}"
              ];
              # TODO(benluelo): refactor into an attrset with name as the key
              extern-paths = [
                { from = ".cosmos.ics23"; to = "::ics23::cosmos::ics23"; name = "ics23"; }
                { from = ".cosmos"; to = "::cosmos_sdk::cosmos"; name = "cosmos-sdk"; }
                { from = ".tendermint"; to = "::cometbls::tendermint"; name = "cometbls"; }
              ];
              # type_attributes = {};
            }
            {
              name = "ics23";
              src = "${ics23}/proto";
              proto-deps = [
                "${ics23}/proto"
              ];
            }
            {
              name = "cosmos-sdk";
              src = "${cosmossdk}/proto/cosmos";
              proto-deps = [
                "${cosmossdk}/proto"
                "${gogoproto}"
                "${cosmosproto}/proto"
                "${googleapis}"
              ];
              extern-paths = [
                { from = ".tendermint"; to = "::cometbls::tendermint"; name = "cometbls"; }
              ];
              # {msg,query}.proto have to be excluded since they only contain extensions, causing the generator to not create a file
              # SEE: https://github.com/neoeinstein/protoc-gen-prost/issues/61
              additional-filter = "-not -path '*cosmos/msg/v1/msg.proto' -not -path '*cosmos/query/v1/query.proto' -and -not -path '*/proto/tendermint/*'";
              fixup-script = ''
                sed -i 's/pub struct Validators/pub struct ValidatorsVec/' "$outdir/src/cosmos.staking.v1beta1.rs"
                sed -i 's/AllowList(Validators)/AllowList(ValidatorsVec)/' "$outdir/src/cosmos.staking.v1beta1.rs"
                sed -i 's/DenyList(Validators)/DenyList(ValidatorsVec)/' "$outdir/src/cosmos.staking.v1beta1.rs"
              '';
            }
            # {
            #   name = "amino";
            #   src = "${cosmossdk}/proto/amino";
            #   proto-deps = [
            #     "${cosmossdk}/proto/amino"
            #     "${cosmosproto}/proto"
            #   ];
            # }
          ];

          type_attributes = with attrs; {
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
            ".ibc.lightclients.ethereum.v1" = [ serde ];
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

          field_attributes = with attrs; {
            ".ibc.core.client.v1.Height" = [ serde_default ];
            ".ibc.core.commitment.v1.MerkleRoot.hash" = [ jsonschema_str serde_base64 ];
            ".ibc.core.commitment.v1.MerklePrefix.key_prefix" = [ jsonschema_str serde_base64 ];
            ".ibc.lightclients.wasm.v1.ClientState.data" = [ serde_base64 ];
            ".ibc.lightclients.wasm.v1.ClientState.code_id" = [ serde_base64 ];
            ".ibc.lightclients.wasm.v1.ConsensusState.data" = [ serde_base64 ];
            ".ibc.lightclients.wasm.v1.Header.data" = [ serde_base64 ];
            ".ibc.lightclients.ethereum.v1.SyncCommittee.aggregate_pubkey" = [ serde_base64 ];
            ".ibc.lightclients.ethereum.v1.SyncCommittee.pubkeys" = [ serde_inner_base64 ];
            ".ibc.lightclients.ethereum.v1.BeaconBlockHeader.parent_root" = [ serde_base64 ];
            ".ibc.lightclients.ethereum.v1.BeaconBlockHeader.state_root" = [ serde_base64 ];
            ".ibc.lightclients.ethereum.v1.BeaconBlockHeader.body_root" = [ serde_base64 ];
            ".ibc.lightclients.ethereum.v1.ExecutionPayloadHeader.parent_hash" = [ serde_base64 ];
            ".ibc.lightclients.ethereum.v1.ExecutionPayloadHeader.fee_recipient" = [ serde_base64 ];
            ".ibc.lightclients.ethereum.v1.ExecutionPayloadHeader.state_root" = [ serde_base64 ];
            ".ibc.lightclients.ethereum.v1.ExecutionPayloadHeader.receipts_root" = [ serde_base64 ];
            ".ibc.lightclients.ethereum.v1.ExecutionPayloadHeader.logs_bloom" = [ serde_base64 ];
            ".ibc.lightclients.ethereum.v1.ExecutionPayloadHeader.prev_randao" = [ serde_base64 ];
            ".ibc.lightclients.ethereum.v1.ExecutionPayloadHeader.extra_data" = [ serde_base64 ];
            ".ibc.lightclients.ethereum.v1.ExecutionPayloadHeader.base_fee_per_gas" = [ serde_base64 ];
            ".ibc.lightclients.ethereum.v1.ExecutionPayloadHeader.block_hash" = [ serde_base64 ];
            ".ibc.lightclients.ethereum.v1.ExecutionPayloadHeader.transactions_root" = [ serde_base64 ];
            ".ibc.lightclients.ethereum.v1.ExecutionPayloadHeader.withdrawals_root" = [ serde_base64 ];
            ".ibc.lightclients.ethereum.v1.LightClientHeader.execution_branch" = [ serde_inner_base64 ];
            ".ibc.lightclients.ethereum.v1.LightClientUpdate.next_sync_committee_branch" = [ serde_inner_base64 ];
            ".ibc.lightclients.ethereum.v1.LightClientUpdate.finality_branch" = [ serde_inner_base64 ];
            ".ibc.lightclients.ethereum.v1.SyncAggregate.sync_committee_bits" = [ serde_base64 ];
            ".ibc.lightclients.ethereum.v1.SyncAggregate.sync_committee_signature" = [ serde_base64 ];
            ".ibc.lightclients.ethereum.v1.AccountUpdate.account_proof" = [ serde_base64 ];
            ".ibc.lightclients.ethereum.v1.AccountUpdate.account_storage_root" = [ serde_base64 ];
            ".cosmos.ics23.v1.LeafOp.prehash_key" = [ serde_default ];
            ".cosmos.ics23.v1.LeafOp.prefix" = [ jsonschema_str serde_base64 ];
            ".cosmos.ics23.v1.InnerOp.prefix" = [ serde_base64 jsonschema_str ];
            ".cosmos.ics23.v1.InnerOp.suffix" = [ serde_base64 jsonschema_str ];
            ".cosmos.ics23.v1.ProofSpec.max_depth" = [ serde_default ];
            ".cosmos.ics23.v1.ProofSpec.min_depth" = [ serde_default ];
            ".cosmos.ics23.v1.InnerSpec.empty_child" = [ serde_default jsonschema_str serde_base64 ];
          };

          to-proto-attrs-opts = { attrset, opt-name }: pkgs.lib.escapeShellArg (pkgs.lib.concatStringsSep "," (pkgs.lib.foldlAttrs (acc: name: value: acc ++ map (attr: "${opt-name}=${name}=${pkgs.lib.escape [","] attr}") value) [ ] attrset));

          gen-proto = { name, src, proto-deps, extern-paths ? [ ], additional-filter ? null, fixup-script ? null }: ''
            outdir=OUT/${name}
            mkdir -p $outdir/src

            echo "[COMPILING] ${name}"

            found="$(find ${src} -type f -name '*.proto' ${if additional-filter != null then "-and " + additional-filter else ""} | tr '\n' ' ')"

            read -ra protos <<<"$found"

            for file in "''${protos[@]}"; do
              echo "[FOUND] $file"
            done

            # echo "PROTOS:"
            # echo "''${protos[@]}"

            protoc "''${protos[@]}" \
              --prost_out="$outdir"/src \
              --prost_opt=compile_well_known_types=true,extern_path=.google.protobuf=::pbjson_types,${
                pkgs.lib.concatStringsSep
                  ","
                  (map
                    ({from, to, ...}: "extern_path=${from}=${to}")
                    extern-paths
                  )
              },${
                to-proto-attrs-opts {
                  attrset = type_attributes;
                  opt-name = "type_attribute";
                }
              },${
                to-proto-attrs-opts {
                  attrset = field_attributes;
                  opt-name = "field_attribute";
                }
              } \
              --tonic_out="$outdir"/src \
              --tonic_opt=compile_well_known_types=true,extern_path=.google.protobuf=::pbjson_types,${pkgs.lib.concatStringsSep "," (map ({from, to, ...}: "extern_path=${from}=${to}") extern-paths)} \
              --prost-crate_out="$outdir" \
              --prost-crate_opt=gen_crate=${cargo_toml { name = name; dependencies = builtins.listToAttrs (map ({name, ...}: { name = name; value = {path = "../${name}";}; }) extern-paths); }} \
              ${build-proto-includes proto-deps}

            ${if fixup-script != null then fixup-script else ""}
          '';


          build-proto-includes = srcs: pkgs.lib.concatStringsSep " " (map (src: ''-I"${src}"'') srcs);
        in
        ''
          mkdir -p OUT

          plugindir=${protoc-gen-tonic}

          export PATH="$plugindir/bin:$PATH"

          ${pkgs.lib.concatStringsSep "\n" (map gen-proto all-proto-build)}
        '';
    };
  };
}


# # rm PROTOCOPTS

# cat >>PROTOCOPTS <<EOL
# ${protoIncludes}
# EOL

# #  >> PROTOCOPTS
# # cat PROTOCOPTS
# # protoc @PROTOCOPTS

# find ${proto.ibcgo}/proto -name "*.proto" |\
# while read -r file; do
#   echo "Generating $file"
#   protoc \
#     ${protoIncludes} \
#     --prost_out=$outdir \
#     --prost-crate_out=include_file=src/lib.rs,gen_crate=$outdir/Cargo.toml:$outdir \
#     --tonic_out=compile_well_known_types=true:$outdir/src \
#     "$file"
# done

# ,extern_path=.tendermint=::tendermint_proto,extern_path=.ics23=::ics23 \

# echo "Generating cometbls protos"

# find ${proto.cometbls}/proto -type f -name "*.proto" |\
# while read -r file; do
#   echo "Generating $file"
#   protoc \
#     ${protoIncludes} \
#     --prost-crate_out=$outdir \
#     --tonic_out=$outdir/src \
#     "$file"
# done

# echo "Generating unionpd protos"

# find ${proto.unionpd} -type f -name "*.proto" |\
# while read -r file; do
#   echo "Generating $file"
#   protoc \
#     ${protoIncludes} \
#     --prost-crate_out=$outdir \
#     --tonic_out=$outdir/src \
#     "$file"
# done

            
