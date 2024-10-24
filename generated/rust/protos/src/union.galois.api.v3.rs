// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FrElement {
    #[prost(bytes = "vec", tag = "1")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for FrElement {
    const NAME: &'static str = "FrElement";
    const PACKAGE: &'static str = "union.galois.api.v3";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.galois.api.v3.{}", Self::NAME)
    }
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
impl ::prost::Name for ZeroKnowledgeProof {
    const NAME: &'static str = "ZeroKnowledgeProof";
    const PACKAGE: &'static str = "union.galois.api.v3";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.galois.api.v3.{}", Self::NAME)
    }
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
impl ::prost::Name for ValidatorSetCommit {
    const NAME: &'static str = "ValidatorSetCommit";
    const PACKAGE: &'static str = "union.galois.api.v3";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.galois.api.v3.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProveRequest {
    #[prost(message, optional, tag = "1")]
    pub vote: ::core::option::Option<super::super::super::super::tendermint::types::CanonicalVote>,
    #[prost(message, optional, tag = "2")]
    pub untrusted_header:
        ::core::option::Option<super::super::super::super::tendermint::types::Header>,
    #[prost(message, optional, tag = "3")]
    pub trusted_commit: ::core::option::Option<ValidatorSetCommit>,
    #[prost(message, optional, tag = "4")]
    pub untrusted_commit: ::core::option::Option<ValidatorSetCommit>,
}
impl ::prost::Name for ProveRequest {
    const NAME: &'static str = "ProveRequest";
    const PACKAGE: &'static str = "union.galois.api.v3";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.galois.api.v3.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProveResponse {
    #[prost(message, optional, tag = "1")]
    pub proof: ::core::option::Option<ZeroKnowledgeProof>,
    #[prost(bytes = "vec", tag = "2")]
    pub trusted_validator_set_root: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ProveResponse {
    const NAME: &'static str = "ProveResponse";
    const PACKAGE: &'static str = "union.galois.api.v3";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.galois.api.v3.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VerifyRequest {
    #[prost(message, optional, tag = "1")]
    pub proof: ::core::option::Option<ZeroKnowledgeProof>,
    #[prost(bytes = "vec", tag = "2")]
    pub inputs_hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for VerifyRequest {
    const NAME: &'static str = "VerifyRequest";
    const PACKAGE: &'static str = "union.galois.api.v3";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.galois.api.v3.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VerifyResponse {
    #[prost(bool, tag = "1")]
    pub valid: bool,
}
impl ::prost::Name for VerifyResponse {
    const NAME: &'static str = "VerifyResponse";
    const PACKAGE: &'static str = "union.galois.api.v3";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.galois.api.v3.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenerateContractRequest {}
impl ::prost::Name for GenerateContractRequest {
    const NAME: &'static str = "GenerateContractRequest";
    const PACKAGE: &'static str = "union.galois.api.v3";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.galois.api.v3.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenerateContractResponse {
    #[prost(bytes = "vec", tag = "1")]
    pub content: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for GenerateContractResponse {
    const NAME: &'static str = "GenerateContractResponse";
    const PACKAGE: &'static str = "union.galois.api.v3";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.galois.api.v3.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryStatsRequest {}
impl ::prost::Name for QueryStatsRequest {
    const NAME: &'static str = "QueryStatsRequest";
    const PACKAGE: &'static str = "union.galois.api.v3";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.galois.api.v3.{}", Self::NAME)
    }
}
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
impl ::prost::Name for VariableStats {
    const NAME: &'static str = "VariableStats";
    const PACKAGE: &'static str = "union.galois.api.v3";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.galois.api.v3.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProvingKeyStats {
    #[prost(uint32, tag = "1")]
    pub nb_g1: u32,
    #[prost(uint32, tag = "2")]
    pub nb_g2: u32,
}
impl ::prost::Name for ProvingKeyStats {
    const NAME: &'static str = "ProvingKeyStats";
    const PACKAGE: &'static str = "union.galois.api.v3";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.galois.api.v3.{}", Self::NAME)
    }
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
impl ::prost::Name for VerifyingKeyStats {
    const NAME: &'static str = "VerifyingKeyStats";
    const PACKAGE: &'static str = "union.galois.api.v3";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.galois.api.v3.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommitmentStats {
    #[prost(uint32, tag = "1")]
    pub nb_public_committed: u32,
    #[prost(uint32, tag = "2")]
    pub nb_private_committed: u32,
}
impl ::prost::Name for CommitmentStats {
    const NAME: &'static str = "CommitmentStats";
    const PACKAGE: &'static str = "union.galois.api.v3";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.galois.api.v3.{}", Self::NAME)
    }
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
impl ::prost::Name for QueryStatsResponse {
    const NAME: &'static str = "QueryStatsResponse";
    const PACKAGE: &'static str = "union.galois.api.v3";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.galois.api.v3.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PollRequest {
    #[prost(message, optional, tag = "1")]
    pub request: ::core::option::Option<ProveRequest>,
}
impl ::prost::Name for PollRequest {
    const NAME: &'static str = "PollRequest";
    const PACKAGE: &'static str = "union.galois.api.v3";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.galois.api.v3.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProveRequestPending {}
impl ::prost::Name for ProveRequestPending {
    const NAME: &'static str = "ProveRequestPending";
    const PACKAGE: &'static str = "union.galois.api.v3";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.galois.api.v3.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProveRequestFailed {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}
impl ::prost::Name for ProveRequestFailed {
    const NAME: &'static str = "ProveRequestFailed";
    const PACKAGE: &'static str = "union.galois.api.v3";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.galois.api.v3.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProveRequestDone {
    #[prost(message, optional, tag = "1")]
    pub response: ::core::option::Option<ProveResponse>,
}
impl ::prost::Name for ProveRequestDone {
    const NAME: &'static str = "ProveRequestDone";
    const PACKAGE: &'static str = "union.galois.api.v3";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.galois.api.v3.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PollResponse {
    #[prost(oneof = "poll_response::Result", tags = "1, 2, 3")]
    pub result: ::core::option::Option<poll_response::Result>,
}
/// Nested message and enum types in `PollResponse`.
pub mod poll_response {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Result {
        #[prost(message, tag = "1")]
        Pending(super::ProveRequestPending),
        #[prost(message, tag = "2")]
        Failed(super::ProveRequestFailed),
        #[prost(message, tag = "3")]
        Done(super::ProveRequestDone),
    }
}
impl ::prost::Name for PollResponse {
    const NAME: &'static str = "PollResponse";
    const PACKAGE: &'static str = "union.galois.api.v3";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.galois.api.v3.{}", Self::NAME)
    }
}
include!("union.galois.api.v3.tonic.rs");
// @@protoc_insertion_point(module)
