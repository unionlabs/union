// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub l1_client_id: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "3")]
    pub l1_contract_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "4")]
    pub l2_contract_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "5")]
    pub table_handle: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "6")]
    pub frozen_height:
        ::core::option::Option<super::super::super::super::super::ibc::core::client::v1::Height>,
    #[prost(uint64, tag = "7")]
    pub latest_block_num: u64,
}
impl ::prost::Name for ClientState {
    const NAME: &'static str = "ClientState";
    const PACKAGE: &'static str = "union.ibc.lightclients.movement.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.movement.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusState {
    /// Aptos state root
    #[prost(bytes = "vec", tag = "1")]
    pub state_root: ::prost::alloc::vec::Vec<u8>,
    /// Movement timestamp
    #[prost(uint64, tag = "2")]
    pub timestamp: u64,
    /// Hash of the `StateProof` which is committed to l1
    #[prost(bytes = "vec", tag = "3")]
    pub state_proof_hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ConsensusState {
    const NAME: &'static str = "ConsensusState";
    const PACKAGE: &'static str = "union.ibc.lightclients.movement.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.movement.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
    #[prost(message, optional, tag = "1")]
    pub l1_height:
        ::core::option::Option<super::super::super::super::super::ibc::core::client::v1::Height>,
    #[prost(message, optional, tag = "2")]
    pub trusted_height:
        ::core::option::Option<super::super::super::super::super::ibc::core::client::v1::Height>,
    #[prost(message, optional, tag = "3")]
    pub state_proof: ::core::option::Option<StateProof>,
    #[prost(uint64, tag = "4")]
    pub tx_index: u64,
    #[prost(message, optional, tag = "5")]
    pub tx_proof: ::core::option::Option<TransactionInfoWithProof>,
    #[prost(message, optional, tag = "6")]
    pub state_proof_hash_proof: ::core::option::Option<super::super::ethereum::v1::StorageProof>,
    #[prost(message, optional, tag = "7")]
    pub settlement_contract_proof: ::core::option::Option<super::super::ethereum::v1::AccountProof>,
    #[prost(uint64, tag = "8")]
    pub new_height: u64,
}
impl ::prost::Name for Header {
    const NAME: &'static str = "Header";
    const PACKAGE: &'static str = "union.ibc.lightclients.movement.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.movement.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionInfoWithProof {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub ledger_info_to_transaction_info_proof:
        ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    #[prost(message, optional, tag = "2")]
    pub transaction_info: ::core::option::Option<TransactionInfo>,
}
impl ::prost::Name for TransactionInfoWithProof {
    const NAME: &'static str = "TransactionInfoWithProof";
    const PACKAGE: &'static str = "union.ibc.lightclients.movement.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.movement.v1.{}", Self::NAME)
    }
}
/// NOTE(aeryz): we don't include status cause we assume success status in the light client. If it's not successfull,
/// the light client will fail since the hash won't match.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionInfo {
    #[prost(uint64, tag = "1")]
    pub gas_used: u64,
    #[prost(bytes = "vec", tag = "2")]
    pub transaction_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub event_root_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "4")]
    pub state_change_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "5")]
    pub state_checkpoint_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "6")]
    pub state_cemetery_hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for TransactionInfo {
    const NAME: &'static str = "TransactionInfo";
    const PACKAGE: &'static str = "union.ibc.lightclients.movement.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.movement.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StateProof {
    #[prost(message, optional, tag = "1")]
    pub latest_li_w_sigs: ::core::option::Option<LedgerInfoWithSignatures>,
    #[prost(message, optional, tag = "2")]
    pub epoch_changes: ::core::option::Option<EpochChangeProof>,
}
impl ::prost::Name for StateProof {
    const NAME: &'static str = "StateProof";
    const PACKAGE: &'static str = "union.ibc.lightclients.movement.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.movement.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LedgerInfoWithSignatures {
    #[prost(message, optional, tag = "1")]
    pub ledger_info: ::core::option::Option<LedgerInfo>,
    /// / Aggregated BLS signature of all the validators that signed the message. The bitmask in the
    /// / aggregated signature can be used to find out the individual validators signing the message
    #[prost(message, optional, tag = "2")]
    pub signatures: ::core::option::Option<AggregateSignature>,
}
impl ::prost::Name for LedgerInfoWithSignatures {
    const NAME: &'static str = "LedgerInfoWithSignatures";
    const PACKAGE: &'static str = "union.ibc.lightclients.movement.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.movement.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LedgerInfo {
    #[prost(message, optional, tag = "1")]
    pub commit_info: ::core::option::Option<BlockInfo>,
    #[prost(bytes = "vec", tag = "2")]
    pub consensus_data_hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for LedgerInfo {
    const NAME: &'static str = "LedgerInfo";
    const PACKAGE: &'static str = "union.ibc.lightclients.movement.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.movement.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockInfo {
    #[prost(uint64, tag = "1")]
    pub epoch: u64,
    #[prost(uint64, tag = "2")]
    pub round: u64,
    #[prost(bytes = "vec", tag = "3")]
    pub id: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "4")]
    pub executed_state_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "5")]
    pub version: u64,
    #[prost(uint64, tag = "6")]
    pub timestamp_usecs: u64,
    #[prost(message, optional, tag = "7")]
    pub next_epoch_state: ::core::option::Option<EpochState>,
}
impl ::prost::Name for BlockInfo {
    const NAME: &'static str = "BlockInfo";
    const PACKAGE: &'static str = "union.ibc.lightclients.movement.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.movement.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EpochState {
    #[prost(uint64, tag = "1")]
    pub epoch: u64,
    #[prost(message, optional, tag = "2")]
    pub verifier: ::core::option::Option<ValidatorVerifier>,
}
impl ::prost::Name for EpochState {
    const NAME: &'static str = "EpochState";
    const PACKAGE: &'static str = "union.ibc.lightclients.movement.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.movement.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorVerifier {
    #[prost(message, repeated, tag = "1")]
    pub validator_infos: ::prost::alloc::vec::Vec<ValidatorConsensusInfo>,
}
impl ::prost::Name for ValidatorVerifier {
    const NAME: &'static str = "ValidatorVerifier";
    const PACKAGE: &'static str = "union.ibc.lightclients.movement.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.movement.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorConsensusInfo {
    #[prost(bytes = "vec", tag = "1")]
    pub address: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub public_key: ::core::option::Option<PublicKey>,
    #[prost(uint64, tag = "3")]
    pub voting_power: u64,
}
impl ::prost::Name for ValidatorConsensusInfo {
    const NAME: &'static str = "ValidatorConsensusInfo";
    const PACKAGE: &'static str = "union.ibc.lightclients.movement.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.movement.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PublicKey {
    #[prost(bytes = "vec", tag = "1")]
    pub pubkey: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for PublicKey {
    const NAME: &'static str = "PublicKey";
    const PACKAGE: &'static str = "union.ibc.lightclients.movement.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.movement.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AggregateSignature {
    #[prost(bytes = "vec", tag = "1")]
    pub validator_bitmask: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub sig: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for AggregateSignature {
    const NAME: &'static str = "AggregateSignature";
    const PACKAGE: &'static str = "union.ibc.lightclients.movement.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.movement.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EpochChangeProof {
    #[prost(message, repeated, tag = "1")]
    pub ledger_info_with_sigs: ::prost::alloc::vec::Vec<LedgerInfoWithSignatures>,
    #[prost(bool, tag = "2")]
    pub more: bool,
}
impl ::prost::Name for EpochChangeProof {
    const NAME: &'static str = "EpochChangeProof";
    const PACKAGE: &'static str = "union.ibc.lightclients.movement.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.movement.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StorageProof {
    #[prost(message, optional, tag = "3")]
    pub proof: ::core::option::Option<SparseMerkleProof>,
    #[prost(oneof = "storage_proof::StateValue", tags = "1, 2")]
    pub state_value: ::core::option::Option<storage_proof::StateValue>,
}
/// Nested message and enum types in `StorageProof`.
pub mod storage_proof {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum StateValue {
        #[prost(bytes, tag = "1")]
        V0(::prost::alloc::vec::Vec<u8>),
        #[prost(message, tag = "2")]
        WithMetadata(super::StateValueWithMetadata),
    }
}
impl ::prost::Name for StorageProof {
    const NAME: &'static str = "StorageProof";
    const PACKAGE: &'static str = "union.ibc.lightclients.movement.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.movement.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StateValueWithMetadata {
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(oneof = "state_value_with_metadata::Metadata", tags = "2, 3")]
    pub metadata: ::core::option::Option<state_value_with_metadata::Metadata>,
}
/// Nested message and enum types in `StateValueWithMetadata`.
pub mod state_value_with_metadata {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Metadata {
        #[prost(message, tag = "2")]
        V0(super::StateValueMetadataV0),
        #[prost(message, tag = "3")]
        V1(super::StateValueMetadataV1),
    }
}
impl ::prost::Name for StateValueWithMetadata {
    const NAME: &'static str = "StateValueWithMetadata";
    const PACKAGE: &'static str = "union.ibc.lightclients.movement.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.movement.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StateValueMetadataV0 {
    #[prost(uint64, tag = "1")]
    pub deposit: u64,
    #[prost(uint64, tag = "2")]
    pub creation_time_usecs: u64,
}
impl ::prost::Name for StateValueMetadataV0 {
    const NAME: &'static str = "StateValueMetadataV0";
    const PACKAGE: &'static str = "union.ibc.lightclients.movement.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.movement.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StateValueMetadataV1 {
    #[prost(uint64, tag = "1")]
    pub slot_deposit: u64,
    #[prost(uint64, tag = "2")]
    pub bytes_deposit: u64,
    #[prost(uint64, tag = "3")]
    pub creation_time_usecs: u64,
}
impl ::prost::Name for StateValueMetadataV1 {
    const NAME: &'static str = "StateValueMetadataV1";
    const PACKAGE: &'static str = "union.ibc.lightclients.movement.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.movement.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SparseMerkleProof {
    #[prost(message, optional, tag = "1")]
    pub leaf: ::core::option::Option<SparseMerkleLeafNode>,
    #[prost(bytes = "vec", repeated, tag = "2")]
    pub siblings: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
impl ::prost::Name for SparseMerkleProof {
    const NAME: &'static str = "SparseMerkleProof";
    const PACKAGE: &'static str = "union.ibc.lightclients.movement.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.movement.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SparseMerkleLeafNode {
    #[prost(bytes = "vec", tag = "1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub value_hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for SparseMerkleLeafNode {
    const NAME: &'static str = "SparseMerkleLeafNode";
    const PACKAGE: &'static str = "union.ibc.lightclients.movement.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.movement.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
