// @generated
/// ConsensusParams contains consensus critical parameters that determine the
/// validity of blocks.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusParams {
    #[prost(message, optional, tag = "1")]
    pub block: ::core::option::Option<BlockParams>,
    #[prost(message, optional, tag = "2")]
    pub evidence: ::core::option::Option<super::v1beta1::EvidenceParams>,
    #[prost(message, optional, tag = "3")]
    pub validator: ::core::option::Option<super::v1beta1::ValidatorParams>,
    #[prost(message, optional, tag = "4")]
    pub version: ::core::option::Option<super::v1beta1::VersionParams>,
}
impl ::prost::Name for ConsensusParams {
    const NAME: &'static str = "ConsensusParams";
    const PACKAGE: &'static str = "cometbft.types.v1beta2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1beta2.{}", Self::NAME)
    }
}
/// BlockParams contains limits on the block size.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockParams {
    /// Max block size, in bytes.
    /// Note: must be greater than 0
    #[prost(int64, tag = "1")]
    pub max_bytes: i64,
    /// Max gas per block.
    /// Note: must be greater or equal to -1
    #[prost(int64, tag = "2")]
    pub max_gas: i64,
}
impl ::prost::Name for BlockParams {
    const NAME: &'static str = "BlockParams";
    const PACKAGE: &'static str = "cometbft.types.v1beta2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1beta2.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
