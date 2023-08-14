// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FrElement {
    #[prost(bytes = "vec", tag = "1")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ZeroKnowledgeProof {
    #[prost(bytes = "vec", tag = "1")]
    pub content: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub compressed_content: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub evm_proof: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "4")]
    pub public_inputs: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorSetCommit {
    #[prost(message, repeated, tag = "1")]
    pub validators:
        ::prost::alloc::vec::Vec<super::super::super::super::tendermint::types::SimpleValidator>,
    #[prost(bytes = "vec", repeated, tag = "2")]
    pub signatures: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    #[prost(bytes = "vec", tag = "3")]
    pub bitmap: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProveRequest {
    #[prost(message, optional, tag = "1")]
    pub vote: ::core::option::Option<super::super::super::super::tendermint::types::CanonicalVote>,
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
    pub trusted_validator_set_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub untrusted_validator_set_root: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VerifyRequest {
    #[prost(message, optional, tag = "1")]
    pub proof: ::core::option::Option<ZeroKnowledgeProof>,
    #[prost(bytes = "vec", tag = "2")]
    pub trusted_validator_set_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub untrusted_validator_set_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "4")]
    pub block_header_x: ::core::option::Option<FrElement>,
    #[prost(message, optional, tag = "5")]
    pub block_header_y: ::core::option::Option<FrElement>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VerifyResponse {
    #[prost(bool, tag = "1")]
    pub valid: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenerateContractRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenerateContractResponse {
    #[prost(bytes = "vec", tag = "1")]
    pub content: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryStatsRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VariableStats {
    #[prost(uint32, tag = "1")]
    pub nb_internal_variables: u32,
    #[prost(uint32, tag = "2")]
    pub nb_secret_variables: u32,
    #[prost(uint32, tag = "3")]
    pub nb_public_variables: u32,
    #[prost(uint32, tag = "4")]
    pub nb_constraints: u32,
    #[prost(uint32, tag = "5")]
    pub nb_coefficients: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProvingKeyStats {
    #[prost(uint32, tag = "1")]
    pub nb_g1: u32,
    #[prost(uint32, tag = "2")]
    pub nb_g2: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VerifyingKeyStats {
    #[prost(uint32, tag = "1")]
    pub nb_g1: u32,
    #[prost(uint32, tag = "2")]
    pub nb_g2: u32,
    #[prost(uint32, tag = "3")]
    pub nb_public_witness: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommitmentStats {
    #[prost(uint32, tag = "1")]
    pub nb_public_committed: u32,
    #[prost(uint32, tag = "2")]
    pub nb_private_committed: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryStatsResponse {
    #[prost(message, optional, tag = "1")]
    pub variable_stats: ::core::option::Option<VariableStats>,
    #[prost(message, optional, tag = "2")]
    pub proving_key_stats: ::core::option::Option<ProvingKeyStats>,
    #[prost(message, optional, tag = "3")]
    pub verifying_key_stats: ::core::option::Option<VerifyingKeyStats>,
    #[prost(message, optional, tag = "4")]
    pub commitment_stats: ::core::option::Option<CommitmentStats>,
}
include!("union.galois.api.v1.tonic.rs");
// @@protoc_insertion_point(module)
