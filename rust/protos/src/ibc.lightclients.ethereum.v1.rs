// @generated
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StorageProof {
    #[prost(message, repeated, tag = "1")]
    pub proof: ::prost::alloc::vec::Vec<Proof>,
}
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Proof {
    #[prost(string, tag = "1")]
    pub key: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub value: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "3")]
    pub proof: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    #[prost(bytes = "vec", tag = "1")]
    pub genesis_validators_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub min_sync_committee_participants: u64,
    #[prost(uint64, tag = "3")]
    pub genesis_time: u64,
    #[prost(message, optional, tag = "4")]
    pub fork_parameters: ::core::option::Option<ForkParameters>,
    #[prost(uint64, tag = "5")]
    pub seconds_per_slot: u64,
    #[prost(uint64, tag = "6")]
    pub slots_per_epoch: u64,
    #[prost(uint64, tag = "7")]
    pub epochs_per_sync_committee_period: u64,
    #[prost(message, optional, tag = "8")]
    pub trust_level: ::core::option::Option<Fraction>,
    #[prost(uint64, tag = "9")]
    pub trusting_period: u64,
    #[prost(uint64, tag = "10")]
    pub latest_slot: u64,
    #[prost(uint64, tag = "11")]
    pub latest_execution_block_number: u64,
    #[prost(message, optional, tag = "12")]
    pub frozen_height: ::core::option::Option<super::super::super::core::client::v1::Height>,
}
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusState {
    #[prost(uint64, tag = "1")]
    pub slot: u64,
    #[prost(bytes = "vec", tag = "2")]
    pub storage_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "3")]
    pub timestamp: u64,
    #[prost(bytes = "vec", tag = "4")]
    pub current_sync_committee: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "5")]
    pub next_sync_committee: ::prost::alloc::vec::Vec<u8>,
}
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
    #[prost(message, optional, tag = "1")]
    pub trusted_sync_committee: ::core::option::Option<TrustedSyncCommittee>,
    #[prost(message, optional, tag = "2")]
    pub consensus_update: ::core::option::Option<LightClientUpdate>,
    #[prost(message, optional, tag = "3")]
    pub account_update: ::core::option::Option<AccountUpdate>,
    #[prost(uint64, tag = "4")]
    pub timestamp: u64,
}
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TrustedSyncCommittee {
    #[prost(message, optional, tag = "1")]
    pub trusted_height: ::core::option::Option<super::super::super::core::client::v1::Height>,
    #[prost(message, optional, tag = "2")]
    pub sync_committee: ::core::option::Option<SyncCommittee>,
    #[prost(bool, tag = "3")]
    pub is_next: bool,
}
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ForkParameters {
    #[prost(bytes = "vec", tag = "1")]
    pub genesis_fork_version: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub genesis_slot: u64,
    #[prost(message, optional, tag = "3")]
    pub altair: ::core::option::Option<Fork>,
    #[prost(message, optional, tag = "4")]
    pub bellatrix: ::core::option::Option<Fork>,
    #[prost(message, optional, tag = "5")]
    pub capella: ::core::option::Option<Fork>,
    #[prost(message, optional, tag = "6")]
    pub eip4844: ::core::option::Option<Fork>,
}
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fraction {
    #[prost(uint64, tag = "1")]
    pub numerator: u64,
    #[prost(uint64, tag = "2")]
    pub denominator: u64,
}
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fork {
    #[prost(bytes = "vec", tag = "1")]
    pub version: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub epoch: u64,
}
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LightClientUpdate {
    #[prost(message, optional, tag = "1")]
    pub attested_header: ::core::option::Option<LightClientHeader>,
    #[prost(message, optional, tag = "2")]
    pub next_sync_committee: ::core::option::Option<SyncCommittee>,
    #[prost(bytes = "vec", repeated, tag = "3")]
    #[serde(with = "::serde_utils::inner_base64")]
    pub next_sync_committee_branch: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    #[prost(message, optional, tag = "4")]
    pub finalized_header: ::core::option::Option<LightClientHeader>,
    #[prost(bytes = "vec", repeated, tag = "5")]
    #[serde(with = "::serde_utils::inner_base64")]
    pub finality_branch: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    #[prost(message, optional, tag = "8")]
    pub sync_aggregate: ::core::option::Option<SyncAggregate>,
    #[prost(uint64, tag = "9")]
    pub signature_slot: u64,
}
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SyncCommittee {
    #[prost(bytes = "vec", repeated, tag = "1")]
    #[serde(with = "::serde_utils::inner_base64")]
    pub pubkeys: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    #[prost(bytes = "vec", tag = "2")]
    #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub aggregate_pubkey: ::prost::alloc::vec::Vec<u8>,
}
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SyncAggregate {
    #[prost(bytes = "vec", tag = "1")]
    #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub sync_committee_bits: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub sync_committee_signature: ::prost::alloc::vec::Vec<u8>,
}
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExecutionUpdate {
    #[prost(bytes = "vec", tag = "1")]
    pub state_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", repeated, tag = "2")]
    pub state_root_branch: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    #[prost(uint64, tag = "3")]
    pub block_number: u64,
    #[prost(bytes = "vec", repeated, tag = "4")]
    pub block_number_branch: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountUpdate {
    #[prost(bytes = "vec", tag = "1")]
    #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub account_proof: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub account_storage_root: ::prost::alloc::vec::Vec<u8>,
}
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LightClientHeader {
    #[prost(message, optional, tag = "1")]
    pub beacon: ::core::option::Option<BeaconBlockHeader>,
    #[prost(message, optional, tag = "2")]
    pub execution: ::core::option::Option<ExecutionPayloadHeader>,
    #[prost(bytes = "vec", repeated, tag = "3")]
    #[serde(with = "::serde_utils::inner_base64")]
    pub execution_branch: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExecutionPayloadHeader {
    #[prost(bytes = "vec", tag = "1")]
    #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub parent_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub fee_recipient: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub state_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "4")]
    #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub receipts_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "5")]
    #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub logs_bloom: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "6")]
    #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub prev_randao: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "7")]
    pub block_number: u64,
    #[prost(uint64, tag = "8")]
    pub gas_limit: u64,
    #[prost(uint64, tag = "9")]
    pub gas_used: u64,
    #[prost(uint64, tag = "10")]
    pub timestamp: u64,
    #[prost(bytes = "vec", tag = "11")]
    #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub extra_data: ::prost::alloc::vec::Vec<u8>,
    /// TODO(aeryz): U256
    #[prost(bytes = "vec", tag = "12")]
    #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub base_fee_per_gas: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "13")]
    #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub block_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "14")]
    #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub transactions_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "15")]
    #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub withdrawals_root: ::prost::alloc::vec::Vec<u8>,
}
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BeaconBlockHeader {
    #[prost(uint64, tag = "1")]
    pub slot: u64,
    #[prost(uint64, tag = "2")]
    pub proposer_index: u64,
    #[prost(bytes = "vec", tag = "3")]
    #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub parent_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "4")]
    #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub state_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "5")]
    #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub body_root: ::prost::alloc::vec::Vec<u8>,
}
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FinalizedHeaderMisbehaviour {
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub trusted_sync_committee: ::core::option::Option<TrustedSyncCommittee>,
    #[prost(message, optional, tag = "3")]
    pub consensus_update_1: ::core::option::Option<LightClientUpdate>,
    #[prost(message, optional, tag = "4")]
    pub consensus_update_2: ::core::option::Option<LightClientUpdate>,
}
#[cfg_attr(feature = "std", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NextSyncCommitteeMisbehaviour {
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub trusted_sync_committee: ::core::option::Option<TrustedSyncCommittee>,
    #[prost(message, optional, tag = "3")]
    pub consensus_update_1: ::core::option::Option<LightClientUpdate>,
    #[prost(message, optional, tag = "4")]
    pub consensus_update_2: ::core::option::Option<LightClientUpdate>,
}
// @@protoc_insertion_point(module)
