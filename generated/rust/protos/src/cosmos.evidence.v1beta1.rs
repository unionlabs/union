// @generated
/// Equivocation implements the Evidence interface and defines evidence of double
/// signing misbehavior.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Equivocation {
    /// height is the equivocation height.
    #[prost(int64, tag = "1")]
    pub height: i64,
    /// time is the equivocation time.
    #[prost(message, optional, tag = "2")]
    pub time: ::core::option::Option<super::super::super::google::protobuf::Timestamp>,
    /// power is the equivocation validator power.
    #[prost(int64, tag = "3")]
    pub power: i64,
    /// consensus_address is the equivocation validator consensus address.
    #[prost(string, tag = "4")]
    pub consensus_address: ::prost::alloc::string::String,
}
impl ::prost::Name for Equivocation {
    const NAME: &'static str = "Equivocation";
    const PACKAGE: &'static str = "cosmos.evidence.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.evidence.v1beta1.{}", Self::NAME)
    }
}
/// GenesisState defines the evidence module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// evidence defines all the evidence at genesis.
    #[prost(message, repeated, tag = "1")]
    pub evidence: ::prost::alloc::vec::Vec<super::super::super::google::protobuf::Any>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "cosmos.evidence.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.evidence.v1beta1.{}", Self::NAME)
    }
}
/// QueryEvidenceRequest is the request type for the Query/Evidence RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryEvidenceRequest {
    /// evidence_hash defines the hash of the requested evidence.
    /// Deprecated: Use hash, a HEX encoded string, instead.
    #[deprecated]
    #[prost(bytes = "vec", tag = "1")]
    pub evidence_hash: ::prost::alloc::vec::Vec<u8>,
    /// hash defines the evidence hash of the requested evidence.
    ///
    /// Since: cosmos-sdk 0.47
    #[prost(string, tag = "2")]
    pub hash: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryEvidenceRequest {
    const NAME: &'static str = "QueryEvidenceRequest";
    const PACKAGE: &'static str = "cosmos.evidence.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.evidence.v1beta1.{}", Self::NAME)
    }
}
/// QueryEvidenceResponse is the response type for the Query/Evidence RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryEvidenceResponse {
    /// evidence returns the requested evidence.
    #[prost(message, optional, tag = "1")]
    pub evidence: ::core::option::Option<super::super::super::google::protobuf::Any>,
}
impl ::prost::Name for QueryEvidenceResponse {
    const NAME: &'static str = "QueryEvidenceResponse";
    const PACKAGE: &'static str = "cosmos.evidence.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.evidence.v1beta1.{}", Self::NAME)
    }
}
/// QueryEvidenceRequest is the request type for the Query/AllEvidence RPC
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAllEvidenceRequest {
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "1")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryAllEvidenceRequest {
    const NAME: &'static str = "QueryAllEvidenceRequest";
    const PACKAGE: &'static str = "cosmos.evidence.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.evidence.v1beta1.{}", Self::NAME)
    }
}
/// QueryAllEvidenceResponse is the response type for the Query/AllEvidence RPC
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAllEvidenceResponse {
    /// evidence returns all evidences.
    #[prost(message, repeated, tag = "1")]
    pub evidence: ::prost::alloc::vec::Vec<super::super::super::google::protobuf::Any>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryAllEvidenceResponse {
    const NAME: &'static str = "QueryAllEvidenceResponse";
    const PACKAGE: &'static str = "cosmos.evidence.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.evidence.v1beta1.{}", Self::NAME)
    }
}
/// MsgSubmitEvidence represents a message that supports submitting arbitrary
/// Evidence of misbehavior such as equivocation or counterfactual signing.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSubmitEvidence {
    /// submitter is the signer account address of evidence.
    #[prost(string, tag = "1")]
    pub submitter: ::prost::alloc::string::String,
    /// evidence defines the evidence of misbehavior.
    #[prost(message, optional, tag = "2")]
    pub evidence: ::core::option::Option<super::super::super::google::protobuf::Any>,
}
impl ::prost::Name for MsgSubmitEvidence {
    const NAME: &'static str = "MsgSubmitEvidence";
    const PACKAGE: &'static str = "cosmos.evidence.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.evidence.v1beta1.{}", Self::NAME)
    }
}
/// MsgSubmitEvidenceResponse defines the Msg/SubmitEvidence response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSubmitEvidenceResponse {
    /// hash defines the hash of the evidence.
    #[prost(bytes = "vec", tag = "4")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgSubmitEvidenceResponse {
    const NAME: &'static str = "MsgSubmitEvidenceResponse";
    const PACKAGE: &'static str = "cosmos.evidence.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.evidence.v1beta1.{}", Self::NAME)
    }
}
include!("cosmos.evidence.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
