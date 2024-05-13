// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub l1_latest_height:
        ::core::option::Option<super::super::super::super::super::ibc::core::client::v1::Height>,
    #[prost(string, tag = "3")]
    pub l1_client_id: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "4")]
    pub l1_rollup_contract_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "5")]
    pub l1_rollup_current_l2_block_number_slot: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "6")]
    pub l1_rollup_current_l2_timestamp_slot: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "7")]
    pub l1_rollup_l2_state_root_hashes_slot: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "8")]
    pub l2_ibc_contract_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "9")]
    pub l2_ibc_contract_commitment_slot: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "10")]
    pub frozen_height:
        ::core::option::Option<super::super::super::super::super::ibc::core::client::v1::Height>,
}
impl ::prost::Name for ClientState {
    const NAME: &'static str = "ClientState";
    const PACKAGE: &'static str = "union.ibc.lightclients.linea.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.linea.v1.{}", Self::NAME)
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
    const PACKAGE: &'static str = "union.ibc.lightclients.linea.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.linea.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
    #[prost(message, optional, tag = "1")]
    pub l1_height:
        ::core::option::Option<super::super::super::super::super::ibc::core::client::v1::Height>,
    #[prost(message, optional, tag = "2")]
    pub l1_rollup_contract_proof: ::core::option::Option<super::super::ethereum::v1::AccountProof>,
    #[prost(uint64, tag = "3")]
    pub l2_block_number: u64,
    #[prost(message, optional, tag = "4")]
    pub l2_block_number_proof: ::core::option::Option<super::super::ethereum::v1::StorageProof>,
    #[prost(bytes = "vec", tag = "5")]
    pub l2_state_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "6")]
    pub l2_state_root_proof: ::core::option::Option<super::super::ethereum::v1::StorageProof>,
    #[prost(uint64, tag = "7")]
    pub l2_timestamp: u64,
    #[prost(message, optional, tag = "8")]
    pub l2_timestamp_proof: ::core::option::Option<super::super::ethereum::v1::StorageProof>,
    #[prost(message, optional, tag = "9")]
    pub l2_ibc_contract_proof: ::core::option::Option<InclusionProof>,
}
impl ::prost::Name for Header {
    const NAME: &'static str = "Header";
    const PACKAGE: &'static str = "union.ibc.lightclients.linea.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.linea.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MerklePath {
    #[prost(bytes = "vec", tag = "1")]
    pub value: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", repeated, tag = "2")]
    pub proof_related_nodes: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
impl ::prost::Name for MerklePath {
    const NAME: &'static str = "MerklePath";
    const PACKAGE: &'static str = "union.ibc.lightclients.linea.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.linea.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InclusionProof {
    #[prost(bytes = "vec", tag = "1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub leaf_index: u64,
    #[prost(message, optional, tag = "3")]
    pub merkle_path: ::core::option::Option<MerklePath>,
}
impl ::prost::Name for InclusionProof {
    const NAME: &'static str = "InclusionProof";
    const PACKAGE: &'static str = "union.ibc.lightclients.linea.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.linea.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NonInclusionProof {
    #[prost(bytes = "vec", tag = "1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub left_leaf_index: u64,
    #[prost(message, optional, tag = "3")]
    pub left_proof: ::core::option::Option<MerklePath>,
    #[prost(uint64, tag = "4")]
    pub right_leaf_index: u64,
    #[prost(message, optional, tag = "5")]
    pub right_proof: ::core::option::Option<MerklePath>,
}
impl ::prost::Name for NonInclusionProof {
    const NAME: &'static str = "NonInclusionProof";
    const PACKAGE: &'static str = "union.ibc.lightclients.linea.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.linea.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MerkleProof {
    #[prost(oneof = "merkle_proof::Proof", tags = "1, 2")]
    pub proof: ::core::option::Option<merkle_proof::Proof>,
}
/// Nested message and enum types in `MerkleProof`.
pub mod merkle_proof {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Proof {
        #[prost(message, tag = "1")]
        Inclusion(super::InclusionProof),
        #[prost(message, tag = "2")]
        Noninclusion(super::NonInclusionProof),
    }
}
impl ::prost::Name for MerkleProof {
    const NAME: &'static str = "MerkleProof";
    const PACKAGE: &'static str = "union.ibc.lightclients.linea.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.linea.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
