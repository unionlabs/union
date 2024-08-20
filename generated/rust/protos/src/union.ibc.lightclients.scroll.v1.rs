// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    #[prost(string, tag = "1")]
    pub l1_client_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub latest_slot: u64,
    #[prost(message, optional, tag = "4")]
    pub frozen_height:
        ::core::option::Option<super::super::super::super::super::ibc::core::client::v1::Height>,
    #[prost(bytes = "vec", tag = "5")]
    pub latest_batch_index_slot: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "6")]
    pub l2_contract_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "7")]
    pub l2_finalized_state_roots_slot: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "8")]
    pub l2_committed_batches_slot: ::prost::alloc::vec::Vec<u8>,
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
    /// Scroll state root
    #[prost(bytes = "vec", tag = "1")]
    pub state_root: ::prost::alloc::vec::Vec<u8>,
    /// Scroll timestamp
    #[prost(uint64, tag = "2")]
    pub timestamp: u64,
    /// IBC stack on Scroll contract storage root
    #[prost(bytes = "vec", tag = "3")]
    pub ibc_storage_root: ::prost::alloc::vec::Vec<u8>,
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
    /// rollupContractOnL1 ∈ L1Stateroot
    #[prost(message, optional, tag = "2")]
    pub l1_account_proof: ::core::option::Option<super::super::ethereum::v1::AccountProof>,
    /// lastBatchIndex ≡ rollupContractOnL1.lastBatchIndex
    #[prost(message, optional, tag = "3")]
    pub last_batch_index_proof: ::core::option::Option<super::super::ethereum::v1::StorageProof>,
    /// L2stateRoot ≡ rollupContractOnL1.finalized\[lastBatchIndex\]
    #[prost(message, optional, tag = "4")]
    pub l2_state_root_proof: ::core::option::Option<super::super::ethereum::v1::StorageProof>,
    /// ibcContractOnL2 ∈ L2StateRoot
    #[prost(message, optional, tag = "5")]
    pub l2_ibc_account_proof: ::core::option::Option<super::super::ethereum::v1::AccountProof>,
    /// batchHash ≡ rollupContractOnL1.batchHashes\[lastBatchIndex\]
    #[prost(message, optional, tag = "6")]
    pub batch_hash_proof: ::core::option::Option<super::super::ethereum::v1::StorageProof>,
    /// The batch header from where we extract the L2 timestamp, then proving:
    /// hash(batchHeader) ≡ batchHash
    #[prost(bytes = "vec", tag = "7")]
    pub batch_header: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for Header {
    const NAME: &'static str = "Header";
    const PACKAGE: &'static str = "union.ibc.lightclients.scroll.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.scroll.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
