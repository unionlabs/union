// @generated
/// QueryParamsRequest defines the request type for querying x/consensus parameters.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
/// QueryParamsResponse defines the response type for querying x/consensus parameters.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// params are the tendermint consensus params stored in the consensus module.
    /// Please note that `params.version` is not populated in this response, it is
    /// tracked separately in the x/upgrade module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<::cometbls::tendermint::types::ConsensusParams>,
}
/// MsgUpdateParams is the Msg/UpdateParams request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParams {
    /// authority is the address that controls the module (defaults to x/gov unless overwritten).
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// params defines the x/consensus parameters to update.
    /// VersionsParams is not included in this Msg because it is tracked
    /// separarately in x/upgrade.
    ///
    /// NOTE: All parameters must be supplied.
    #[prost(message, optional, tag = "2")]
    pub block: ::core::option::Option<::cometbls::tendermint::types::BlockParams>,
    #[prost(message, optional, tag = "3")]
    pub evidence: ::core::option::Option<::cometbls::tendermint::types::EvidenceParams>,
    #[prost(message, optional, tag = "4")]
    pub validator: ::core::option::Option<
        ::cometbls::tendermint::types::ValidatorParams,
    >,
}
/// MsgUpdateParamsResponse defines the response structure for executing a
/// MsgUpdateParams message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParamsResponse {}
include!("cosmos.consensus.v1.tonic.rs");
// @@protoc_insertion_point(module)
