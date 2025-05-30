/// GetBlockIndexerRetainHeightRequest is a request for the retain height.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct GetBlockIndexerRetainHeightRequest {}
/// GetBlockIndexerRetainHeightResponse returns the retain height for the block indexer.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct GetBlockIndexerRetainHeightResponse {
    #[prost(uint64, tag = "1")]
    pub height: u64,
}
/// GetBlockResultsRetainHeightRequest is a request for the retain height.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct GetBlockResultsRetainHeightRequest {}
/// GetBlockResultsRetainHeightResponse returns the retain height for block results.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct GetBlockResultsRetainHeightResponse {
    /// The retain height set by the pruning service (e.g. by the data
    /// companion) specifically for block results.
    #[prost(uint64, tag = "1")]
    pub pruning_service_retain_height: u64,
}
/// GetBlockRetainHeightRequest is a request for the retain height.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct GetBlockRetainHeightRequest {}
/// GetBlockRetainHeightResponse returns the retain height for blocks.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct GetBlockRetainHeightResponse {
    /// The retain height set by the application.
    #[prost(uint64, tag = "1")]
    pub app_retain_height: u64,
    /// The retain height set via the pruning service (e.g. by the data
    /// companion) specifically for blocks.
    #[prost(uint64, tag = "2")]
    pub pruning_service_retain_height: u64,
}
/// GetTxIndexerRetainHeightRequest is a request for the retain height.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct GetTxIndexerRetainHeightRequest {}
/// GetTxIndexerRetainHeightResponse returns the retain height for the tx indexer.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct GetTxIndexerRetainHeightResponse {
    #[prost(uint64, tag = "1")]
    pub height: u64,
}
/// SetBlockIndexerRetainHeightRequest sets the retain height for the block indexer.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct SetBlockIndexerRetainHeightRequest {
    #[prost(uint64, tag = "1")]
    pub height: u64,
}
/// SetBlockIndexerRetainHeightResponse is empty.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct SetBlockIndexerRetainHeightResponse {}
/// SetBlockResultsRetainHeightRequest sets the retain height for block results.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct SetBlockResultsRetainHeightRequest {
    #[prost(uint64, tag = "1")]
    pub height: u64,
}
/// SetBlockResultsRetainHeightResponse is empty.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct SetBlockResultsRetainHeightResponse {}
/// SetBlockRetainHeightRequest sets the retain height for blocks.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct SetBlockRetainHeightRequest {
    #[prost(uint64, tag = "1")]
    pub height: u64,
}
/// SetBlockRetainHeightResponse is empty.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct SetBlockRetainHeightResponse {}
/// SetTxIndexerRetainHeightRequest sets the retain height for the tx indexer.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct SetTxIndexerRetainHeightRequest {
    #[prost(uint64, tag = "1")]
    pub height: u64,
}
/// SetTxIndexerRetainHeightResponse is empty.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct SetTxIndexerRetainHeightResponse {}
impl ::prost::Name for GetBlockIndexerRetainHeightRequest {
    const NAME: &'static str = "GetBlockIndexerRetainHeightRequest";
    const PACKAGE: &'static str = "cometbft.services.pruning.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.services.pruning.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for GetBlockIndexerRetainHeightResponse {
    const NAME: &'static str = "GetBlockIndexerRetainHeightResponse";
    const PACKAGE: &'static str = "cometbft.services.pruning.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.services.pruning.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for GetBlockResultsRetainHeightRequest {
    const NAME: &'static str = "GetBlockResultsRetainHeightRequest";
    const PACKAGE: &'static str = "cometbft.services.pruning.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.services.pruning.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for GetBlockResultsRetainHeightResponse {
    const NAME: &'static str = "GetBlockResultsRetainHeightResponse";
    const PACKAGE: &'static str = "cometbft.services.pruning.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.services.pruning.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for GetBlockRetainHeightRequest {
    const NAME: &'static str = "GetBlockRetainHeightRequest";
    const PACKAGE: &'static str = "cometbft.services.pruning.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.services.pruning.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for GetBlockRetainHeightResponse {
    const NAME: &'static str = "GetBlockRetainHeightResponse";
    const PACKAGE: &'static str = "cometbft.services.pruning.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.services.pruning.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for GetTxIndexerRetainHeightRequest {
    const NAME: &'static str = "GetTxIndexerRetainHeightRequest";
    const PACKAGE: &'static str = "cometbft.services.pruning.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.services.pruning.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for GetTxIndexerRetainHeightResponse {
    const NAME: &'static str = "GetTxIndexerRetainHeightResponse";
    const PACKAGE: &'static str = "cometbft.services.pruning.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.services.pruning.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for SetBlockIndexerRetainHeightRequest {
    const NAME: &'static str = "SetBlockIndexerRetainHeightRequest";
    const PACKAGE: &'static str = "cometbft.services.pruning.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.services.pruning.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for SetBlockIndexerRetainHeightResponse {
    const NAME: &'static str = "SetBlockIndexerRetainHeightResponse";
    const PACKAGE: &'static str = "cometbft.services.pruning.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.services.pruning.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for SetBlockResultsRetainHeightRequest {
    const NAME: &'static str = "SetBlockResultsRetainHeightRequest";
    const PACKAGE: &'static str = "cometbft.services.pruning.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.services.pruning.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for SetBlockResultsRetainHeightResponse {
    const NAME: &'static str = "SetBlockResultsRetainHeightResponse";
    const PACKAGE: &'static str = "cometbft.services.pruning.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.services.pruning.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for SetBlockRetainHeightRequest {
    const NAME: &'static str = "SetBlockRetainHeightRequest";
    const PACKAGE: &'static str = "cometbft.services.pruning.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.services.pruning.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for SetBlockRetainHeightResponse {
    const NAME: &'static str = "SetBlockRetainHeightResponse";
    const PACKAGE: &'static str = "cometbft.services.pruning.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.services.pruning.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for SetTxIndexerRetainHeightRequest {
    const NAME: &'static str = "SetTxIndexerRetainHeightRequest";
    const PACKAGE: &'static str = "cometbft.services.pruning.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.services.pruning.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for SetTxIndexerRetainHeightResponse {
    const NAME: &'static str = "SetTxIndexerRetainHeightResponse";
    const PACKAGE: &'static str = "cometbft.services.pruning.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.services.pruning.v1.{}", Self::NAME)
    }
}
