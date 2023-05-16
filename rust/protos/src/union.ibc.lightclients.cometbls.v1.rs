// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub trust_level: ::core::option::Option<Fraction>,
    /// duration of the period since the LastestTimestamp during which the
    /// submitted headers are valid for upgrade
    #[prost(message, optional, tag = "3")]
    pub trusting_period: ::core::option::Option<
        super::super::super::super::super::google::protobuf::Duration,
    >,
    /// duration of the staking unbonding period
    #[prost(message, optional, tag = "4")]
    pub unbonding_period: ::core::option::Option<
        super::super::super::super::super::google::protobuf::Duration,
    >,
    /// defines how much new (untrusted) header's Time can drift into the future.
    #[prost(message, optional, tag = "5")]
    pub max_clock_drift: ::core::option::Option<
        super::super::super::super::super::google::protobuf::Duration,
    >,
    /// Block height when the client was frozen due to a misbehaviour
    #[prost(message, optional, tag = "6")]
    pub frozen_height: ::core::option::Option<
        super::super::super::super::super::ibc::core::client::v1::Height,
    >,
    /// Latest height the client was updated to
    #[prost(message, optional, tag = "7")]
    pub latest_height: ::core::option::Option<
        super::super::super::super::super::ibc::core::client::v1::Height,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusState {
    /// timestamp that corresponds to the block height in which the ConsensusState
    /// was stored.
    #[prost(message, optional, tag = "1")]
    pub timestamp: ::core::option::Option<
        super::super::super::super::super::google::protobuf::Timestamp,
    >,
    /// commitment root (i.e app hash)
    #[prost(message, optional, tag = "2")]
    pub root: ::core::option::Option<
        super::super::super::super::super::ibc::core::commitment::v1::MerkleRoot,
    >,
    #[prost(bytes = "vec", tag = "3")]
    pub next_validators_hash: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Misbehaviour {
    #[prost(message, optional, tag = "2")]
    pub header_1: ::core::option::Option<Header>,
    #[prost(message, optional, tag = "3")]
    pub header_2: ::core::option::Option<Header>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
    #[prost(message, optional, tag = "1")]
    pub signed_header: ::core::option::Option<
        super::super::super::super::super::tendermint::types::SignedHeader,
    >,
    #[prost(bytes = "vec", tag = "2")]
    pub untrusted_validator_set_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub trusted_height: ::core::option::Option<
        super::super::super::super::super::ibc::core::client::v1::Height,
    >,
    #[prost(bytes = "vec", tag = "4")]
    pub zero_knowledge_proof: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fraction {
    #[prost(uint64, tag = "1")]
    pub numerator: u64,
    #[prost(uint64, tag = "2")]
    pub denominator: u64,
}
// @@protoc_insertion_point(module)
