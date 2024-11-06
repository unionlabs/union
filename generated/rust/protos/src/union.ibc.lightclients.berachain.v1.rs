// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    #[prost(string, tag = "1")]
    pub consensus_chain_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub execution_chain_id: ::prost::alloc::string::String,
    /// TENDERMINT
    #[prost(message, optional, tag = "3")]
    pub trust_level: ::core::option::Option<
        super::super::super::super::super::ibc::lightclients::tendermint::v1::Fraction,
    >,
    #[prost(message, optional, tag = "4")]
    pub trusting_period: ::core::option::Option<::pbjson_types::Duration>,
    #[prost(message, optional, tag = "5")]
    pub max_clock_drift: ::core::option::Option<::pbjson_types::Duration>,
    #[prost(message, optional, tag = "6")]
    pub frozen_height:
        ::core::option::Option<super::super::super::super::super::ibc::core::client::v1::Height>,
    #[prost(message, optional, tag = "7")]
    pub latest_height:
        ::core::option::Option<super::super::super::super::super::ibc::core::client::v1::Height>,
    #[prost(message, repeated, tag = "8")]
    pub proof_specs:
        ::prost::alloc::vec::Vec<super::super::super::super::super::cosmos::ics23::v1::ProofSpec>,
    #[prost(string, repeated, tag = "9")]
    pub upgrade_path: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// ETHEREUM
    #[prost(bytes = "vec", tag = "10")]
    pub ibc_contract_address: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ClientState {
    const NAME: &'static str = "ClientState";
    const PACKAGE: &'static str = "union.ibc.lightclients.berachain.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.berachain.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusState {
    #[prost(uint64, tag = "1")]
    pub eth_timestamp: u64,
    #[prost(message, optional, tag = "2")]
    pub comet_timestamp: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(bytes = "vec", tag = "3")]
    pub eth_storage_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "4")]
    pub comet_next_validators_hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ConsensusState {
    const NAME: &'static str = "ConsensusState";
    const PACKAGE: &'static str = "union.ibc.lightclients.berachain.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.berachain.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
    /// Full cometbft header.
    #[prost(message, optional, tag = "1")]
    pub cometbft_header: ::core::option::Option<
        super::super::super::super::super::ibc::lightclients::tendermint::v1::Header,
    >,
    /// Latest execution header stored in the beacon state.
    #[prost(message, optional, tag = "2")]
    pub execution_header:
        ::core::option::Option<super::super::ethereum::v1::ExecutionPayloadHeader>,
    /// Proof of the latest execution header stored in the beacon state.
    #[prost(message, optional, tag = "3")]
    pub execution_header_proof: ::core::option::Option<
        super::super::super::super::super::ibc::core::commitment::v1::MerkleProof,
    >,
    /// Proof of the ibc contract in the evm state root.
    #[prost(message, optional, tag = "4")]
    pub account_proof: ::core::option::Option<super::super::ethereum::v1::AccountProof>,
}
impl ::prost::Name for Header {
    const NAME: &'static str = "Header";
    const PACKAGE: &'static str = "union.ibc.lightclients.berachain.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.berachain.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
