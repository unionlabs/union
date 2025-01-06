// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// duration of the period since the LastestTimestamp during which the
    /// submitted headers are valid for upgrade
    #[prost(uint64, tag = "2")]
    pub trusting_period: u64,
    /// defines how much new (untrusted) header's Time can drift into the future.
    #[prost(uint64, tag = "4")]
    pub max_clock_drift: u64,
    /// Block height when the client was frozen due to a misbehaviour
    #[prost(message, optional, tag = "5")]
    pub frozen_height:
        ::core::option::Option<super::super::super::super::super::ibc::core::client::v1::Height>,
    /// Latest height the client was updated to
    #[prost(message, optional, tag = "6")]
    pub latest_height:
        ::core::option::Option<super::super::super::super::super::ibc::core::client::v1::Height>,
}
impl ::prost::Name for ClientState {
    const NAME: &'static str = "ClientState";
    const PACKAGE: &'static str = "union.ibc.lightclients.cometbls.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.cometbls.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusState {
    /// timestamp that corresponds to the block height in which the ConsensusState
    /// was stored.
    #[prost(uint64, tag = "1")]
    pub timestamp: u64,
    /// commitment root (i.e app hash)
    #[prost(message, optional, tag = "2")]
    pub root: ::core::option::Option<
        super::super::super::super::super::ibc::core::commitment::v1::MerkleRoot,
    >,
    #[prost(bytes = "vec", tag = "3")]
    pub next_validators_hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ConsensusState {
    const NAME: &'static str = "ConsensusState";
    const PACKAGE: &'static str = "union.ibc.lightclients.cometbls.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.cometbls.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Misbehaviour {
    #[prost(message, optional, tag = "1")]
    pub header_a: ::core::option::Option<Header>,
    #[prost(message, optional, tag = "2")]
    pub header_b: ::core::option::Option<Header>,
}
impl ::prost::Name for Misbehaviour {
    const NAME: &'static str = "Misbehaviour";
    const PACKAGE: &'static str = "union.ibc.lightclients.cometbls.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.cometbls.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LightHeader {
    #[prost(int64, tag = "1")]
    pub height: i64,
    #[prost(message, optional, tag = "2")]
    pub time: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(bytes = "vec", tag = "3")]
    pub validators_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "4")]
    pub next_validators_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "5")]
    pub app_hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for LightHeader {
    const NAME: &'static str = "LightHeader";
    const PACKAGE: &'static str = "union.ibc.lightclients.cometbls.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.cometbls.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
    #[prost(message, optional, tag = "1")]
    pub signed_header: ::core::option::Option<LightHeader>,
    #[prost(message, optional, tag = "2")]
    pub trusted_height:
        ::core::option::Option<super::super::super::super::super::ibc::core::client::v1::Height>,
    #[prost(bytes = "vec", tag = "3")]
    pub zero_knowledge_proof: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for Header {
    const NAME: &'static str = "Header";
    const PACKAGE: &'static str = "union.ibc.lightclients.cometbls.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.cometbls.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
