// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ScrollFinalizedProof {
    #[prost(uint64, tag = "1")]
    pub batch_index: u64,
    #[prost(bytes = "vec", tag = "2")]
    pub finalized_state_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", repeated, tag = "3")]
    pub proof: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    #[prost(string, tag = "1")]
    pub l1_client_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub latest_batch_index: u64,
    #[prost(message, optional, tag = "4")]
    pub frozen_height:
        ::core::option::Option<super::super::super::super::super::ibc::core::client::v1::Height>,
    #[prost(bytes = "vec", tag = "5")]
    pub rollup_contract_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "6")]
    pub rollup_finalized_state_roots_slot: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "7")]
    pub ibc_contract_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "8")]
    pub ibc_commitment_slot: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusState {
    #[prost(uint64, tag = "1")]
    pub batch_index: u64,
    #[prost(bytes = "vec", tag = "2")]
    pub ibc_storage_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "3")]
    pub timestamp: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
    #[prost(message, optional, tag = "1")]
    pub l1_height:
        ::core::option::Option<super::super::super::super::super::ibc::core::client::v1::Height>,
    #[prost(message, optional, tag = "2")]
    pub l1_account_proof: ::core::option::Option<super::super::ethereum::v1::AccountProof>,
    #[prost(message, optional, tag = "3")]
    pub finalized_proof: ::core::option::Option<ScrollFinalizedProof>,
    #[prost(message, optional, tag = "4")]
    pub ibc_account_proof: ::core::option::Option<super::super::ethereum::v1::AccountProof>,
}
// @@protoc_insertion_point(module)
