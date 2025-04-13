// @generated
/// GenesisState defines the monitor module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "babylon.monitor.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.monitor.v1.{}", Self::NAME)
    }
}
/// QueryEndedEpochBtcHeightRequest defines a query type for EndedEpochBtcHeight
/// RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryEndedEpochBtcHeightRequest {
    #[prost(uint64, tag = "1")]
    pub epoch_num: u64,
}
impl ::prost::Name for QueryEndedEpochBtcHeightRequest {
    const NAME: &'static str = "QueryEndedEpochBtcHeightRequest";
    const PACKAGE: &'static str = "babylon.monitor.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.monitor.v1.{}", Self::NAME)
    }
}
/// QueryEndedEpochBtcHeightResponse defines a response type for
/// EndedEpochBtcHeight RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryEndedEpochBtcHeightResponse {
    /// height of btc light client when epoch ended
    #[prost(uint32, tag = "1")]
    pub btc_light_client_height: u32,
}
impl ::prost::Name for QueryEndedEpochBtcHeightResponse {
    const NAME: &'static str = "QueryEndedEpochBtcHeightResponse";
    const PACKAGE: &'static str = "babylon.monitor.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.monitor.v1.{}", Self::NAME)
    }
}
/// QueryReportedCheckpointBtcHeightRequest defines a query type for
/// ReportedCheckpointBtcHeight RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryReportedCheckpointBtcHeightRequest {
    /// ckpt_hash is hex encoded byte string of the hash of the checkpoint
    #[prost(string, tag = "1")]
    pub ckpt_hash: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryReportedCheckpointBtcHeightRequest {
    const NAME: &'static str = "QueryReportedCheckpointBtcHeightRequest";
    const PACKAGE: &'static str = "babylon.monitor.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.monitor.v1.{}", Self::NAME)
    }
}
/// QueryReportedCheckpointBtcHeightResponse defines a response type for
/// ReportedCheckpointBtcHeight RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryReportedCheckpointBtcHeightResponse {
    /// height of btc light client when checkpoint is reported
    #[prost(uint32, tag = "1")]
    pub btc_light_client_height: u32,
}
impl ::prost::Name for QueryReportedCheckpointBtcHeightResponse {
    const NAME: &'static str = "QueryReportedCheckpointBtcHeightResponse";
    const PACKAGE: &'static str = "babylon.monitor.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.monitor.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
