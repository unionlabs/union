// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    #[prost(string, tag = "1")]
    pub l1_client_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub l2_client_id: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub latest_slot: u64,
    /// Evm
    #[prost(bytes = "vec", tag = "5")]
    pub ibc_commitment_slot: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "6")]
    pub ibc_contract_address: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ClientState {
    const NAME: &'static str = "ClientState";
    const PACKAGE: &'static str = "union.ibc.lightclients.evmincosmos.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.evmincosmos.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusState {
    #[prost(bytes = "vec", tag = "1")]
    pub evm_state_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub ibc_storage_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "3")]
    pub timestamp: u64,
}
impl ::prost::Name for ConsensusState {
    const NAME: &'static str = "ConsensusState";
    const PACKAGE: &'static str = "union.ibc.lightclients.evmincosmos.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.evmincosmos.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
    #[prost(message, optional, tag = "1")]
    pub l1_height:
        ::core::option::Option<super::super::super::super::super::ibc::core::client::v1::Height>,
    #[prost(uint64, tag = "2")]
    pub l2_slot: u64,
    #[prost(message, optional, tag = "3")]
    pub l2_consensus_state: ::core::option::Option<super::super::ethereum::v1::ConsensusState>,
    /// Proof of the l2 consensus state in the l1 client.
    #[prost(message, optional, tag = "4")]
    pub l2_inclusion_proof: ::core::option::Option<
        super::super::super::super::super::ibc::core::commitment::v1::MerkleProof,
    >,
    /// Proof of the ibc contract in the evm state root.
    #[prost(message, optional, tag = "5")]
    pub account_proof: ::core::option::Option<super::super::ethereum::v1::AccountProof>,
}
impl ::prost::Name for Header {
    const NAME: &'static str = "Header";
    const PACKAGE: &'static str = "union.ibc.lightclients.evmincosmos.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.evmincosmos.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
