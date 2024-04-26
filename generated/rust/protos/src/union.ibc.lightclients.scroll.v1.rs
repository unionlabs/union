// @generated
/// TODO: l2_ instead of rollup_
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    #[prost(string, tag = "1")]
    pub l1_client_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub latest_batch_index: u64,
    /// TODO: Should be rollup_
    #[prost(bytes = "vec", tag = "4")]
    pub latest_batch_index_slot: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "5")]
    pub frozen_height:
        ::core::option::Option<super::super::super::super::super::ibc::core::client::v1::Height>,
    #[prost(bytes = "vec", tag = "6")]
    pub rollup_contract_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "7")]
    pub rollup_finalized_state_roots_slot: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "8")]
    pub rollup_committed_batches_slot: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "9")]
    pub ibc_contract_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "10")]
    pub ibc_commitment_slot: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ClientState {
    const NAME: &'static str = "ClientState";
    const PACKAGE: &'static str = "union.ibc.lightclients.scroll.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.scroll.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusState {
    #[prost(bytes = "vec", tag = "1")]
    pub ibc_storage_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub timestamp: u64,
}
impl ::prost::Name for ConsensusState {
    const NAME: &'static str = "ConsensusState";
    const PACKAGE: &'static str = "union.ibc.lightclients.scroll.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.scroll.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
    #[prost(message, optional, tag = "1")]
    pub l1_height:
        ::core::option::Option<super::super::super::super::super::ibc::core::client::v1::Height>,
    #[prost(message, optional, tag = "2")]
    pub l1_account_proof: ::core::option::Option<super::super::ethereum::v1::AccountProof>,
    #[prost(bytes = "vec", tag = "3")]
    pub l2_state_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "4")]
    pub l2_state_proof: ::core::option::Option<super::super::ethereum::v1::StorageProof>,
    #[prost(uint64, tag = "5")]
    pub last_batch_index: u64,
    #[prost(message, optional, tag = "6")]
    pub last_batch_index_proof: ::core::option::Option<super::super::ethereum::v1::StorageProof>,
    #[prost(message, optional, tag = "7")]
    pub batch_hash_proof: ::core::option::Option<super::super::ethereum::v1::StorageProof>,
    #[prost(message, optional, tag = "8")]
    pub l2_ibc_account_proof: ::core::option::Option<super::super::ethereum::v1::AccountProof>,
    #[prost(bytes = "vec", tag = "9")]
    pub commit_batch_calldata: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, repeated, tag = "10")]
    pub l1_message_hashes: ::prost::alloc::vec::Vec<IdentifiedL1MessageHash>,
    #[prost(bytes = "vec", tag = "11")]
    pub blob_versioned_hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for Header {
    const NAME: &'static str = "Header";
    const PACKAGE: &'static str = "union.ibc.lightclients.scroll.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.scroll.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IdentifiedL1MessageHash {
    #[prost(uint64, tag = "1")]
    pub queue_index: u64,
    #[prost(bytes = "vec", tag = "2")]
    pub message_hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for IdentifiedL1MessageHash {
    const NAME: &'static str = "IdentifiedL1MessageHash";
    const PACKAGE: &'static str = "union.ibc.lightclients.scroll.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.scroll.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
