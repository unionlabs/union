// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    #[prost(string, tag = "1")]
    pub l2_chain_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub l1_client_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub l2_client_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "4")]
    pub latest_height:
        ::core::option::Option<super::super::super::super::super::ibc::core::client::v1::Height>,
}
impl ::prost::Name for ClientState {
    const NAME: &'static str = "ClientState";
    const PACKAGE: &'static str = "union.ibc.lightclients.cosmosincosmos.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.cosmosincosmos.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusState {
    /// timestamp that corresponds to the block height in which the ConsensusState
    /// was stored.
    #[prost(uint64, tag = "1")]
    pub timestamp: u64,
    /// commitment root (app_hash)
    #[prost(message, optional, tag = "2")]
    pub app_hash: ::core::option::Option<
        super::super::super::super::super::ibc::core::commitment::v1::MerkleRoot,
    >,
}
impl ::prost::Name for ConsensusState {
    const NAME: &'static str = "ConsensusState";
    const PACKAGE: &'static str = "union.ibc.lightclients.cosmosincosmos.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.cosmosincosmos.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
    #[prost(message, optional, tag = "1")]
    pub l1_height:
        ::core::option::Option<super::super::super::super::super::ibc::core::client::v1::Height>,
    #[prost(message, optional, tag = "2")]
    pub l2_height:
        ::core::option::Option<super::super::super::super::super::ibc::core::client::v1::Height>,
    /// Eth abi encoded ICS23 MerkleProof
    #[prost(bytes = "vec", tag = "3")]
    pub l2_inclusion_proof: ::prost::alloc::vec::Vec<u8>,
    /// Proto encoded tendermint ConsensusState
    /// Optimize: technically extractable from the merkleproof as it must be an inclusion proof.
    #[prost(bytes = "vec", tag = "4")]
    pub l2_consensus_state: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for Header {
    const NAME: &'static str = "Header";
    const PACKAGE: &'static str = "union.ibc.lightclients.cosmosincosmos.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.cosmosincosmos.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
