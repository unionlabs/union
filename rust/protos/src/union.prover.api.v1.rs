// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ZeroKnowledgeProof {
    #[prost(bytes = "vec", tag = "1")]
    pub content: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorSetCommit {
    #[prost(message, repeated, tag = "1")]
    pub validators: ::prost::alloc::vec::Vec<
        super::super::super::super::tendermint::types::SimpleValidator,
    >,
    #[prost(bytes = "vec", repeated, tag = "2")]
    pub signatures: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    #[prost(bytes = "vec", tag = "3")]
    pub bitmap: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProveRequest {
    #[prost(message, optional, tag = "1")]
    pub vote: ::core::option::Option<
        super::super::super::super::tendermint::types::CanonicalVote,
    >,
    #[prost(message, optional, tag = "2")]
    pub trusted_commit: ::core::option::Option<ValidatorSetCommit>,
    #[prost(message, optional, tag = "3")]
    pub untrusted_commit: ::core::option::Option<ValidatorSetCommit>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProveResponse {
    #[prost(message, optional, tag = "1")]
    pub proof: ::core::option::Option<ZeroKnowledgeProof>,
    #[prost(bytes = "vec", tag = "2")]
    pub untrusted_validator_set_root: ::prost::alloc::vec::Vec<u8>,
}
include!("union.prover.api.v1.tonic.rs");
// @@protoc_insertion_point(module)
