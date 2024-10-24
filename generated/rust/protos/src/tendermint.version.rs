// @generated
/// App includes the protocol and software version for the application.
/// This information is included in ResponseInfo. The App.Protocol can be
/// updated in ResponseEndBlock.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct App {
    #[prost(uint64, tag = "1")]
    pub protocol: u64,
    #[prost(string, tag = "2")]
    pub software: ::prost::alloc::string::String,
}
impl ::prost::Name for App {
    const NAME: &'static str = "App";
    const PACKAGE: &'static str = "tendermint.version";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.version.{}", Self::NAME)
    }
}
/// Consensus captures the consensus rules for processing a block in the blockchain,
/// including all blockchain data structures and the rules of the application's
/// state transition machine.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Consensus {
    #[prost(uint64, tag = "1")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub block: u64,
    #[prost(uint64, tag = "2")]
    #[cfg_attr(feature = "serde", serde(default))]
    pub app: u64,
}
impl ::prost::Name for Consensus {
    const NAME: &'static str = "Consensus";
    const PACKAGE: &'static str = "tendermint.version";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.version.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
