// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RemoteSignerError {
    #[prost(int32, tag = "1")]
    pub code: i32,
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
}
impl ::prost::Name for RemoteSignerError {
    const NAME: &'static str = "RemoteSignerError";
    const PACKAGE: &'static str = "tendermint.privval";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.privval.{}", Self::NAME)
    }
}
/// PubKeyRequest requests the consensus public key from the remote signer.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PubKeyRequest {
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
}
impl ::prost::Name for PubKeyRequest {
    const NAME: &'static str = "PubKeyRequest";
    const PACKAGE: &'static str = "tendermint.privval";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.privval.{}", Self::NAME)
    }
}
/// PubKeyResponse is a response message containing the public key.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PubKeyResponse {
    #[prost(message, optional, tag = "1")]
    pub pub_key: ::core::option::Option<super::crypto::PublicKey>,
    #[prost(message, optional, tag = "2")]
    pub error: ::core::option::Option<RemoteSignerError>,
}
impl ::prost::Name for PubKeyResponse {
    const NAME: &'static str = "PubKeyResponse";
    const PACKAGE: &'static str = "tendermint.privval";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.privval.{}", Self::NAME)
    }
}
/// SignVoteRequest is a request to sign a vote
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignVoteRequest {
    #[prost(message, optional, tag = "1")]
    pub vote: ::core::option::Option<super::types::Vote>,
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
    /// if true, the signer may skip signing the extension bytes.
    #[prost(bool, tag = "3")]
    pub skip_extension_signing: bool,
}
impl ::prost::Name for SignVoteRequest {
    const NAME: &'static str = "SignVoteRequest";
    const PACKAGE: &'static str = "tendermint.privval";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.privval.{}", Self::NAME)
    }
}
/// SignedVoteResponse is a response containing a signed vote or an error
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignedVoteResponse {
    #[prost(message, optional, tag = "1")]
    pub vote: ::core::option::Option<super::types::Vote>,
    #[prost(message, optional, tag = "2")]
    pub error: ::core::option::Option<RemoteSignerError>,
}
impl ::prost::Name for SignedVoteResponse {
    const NAME: &'static str = "SignedVoteResponse";
    const PACKAGE: &'static str = "tendermint.privval";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.privval.{}", Self::NAME)
    }
}
/// SignProposalRequest is a request to sign a proposal
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignProposalRequest {
    #[prost(message, optional, tag = "1")]
    pub proposal: ::core::option::Option<super::types::Proposal>,
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
}
impl ::prost::Name for SignProposalRequest {
    const NAME: &'static str = "SignProposalRequest";
    const PACKAGE: &'static str = "tendermint.privval";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.privval.{}", Self::NAME)
    }
}
/// SignedProposalResponse is response containing a signed proposal or an error
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignedProposalResponse {
    #[prost(message, optional, tag = "1")]
    pub proposal: ::core::option::Option<super::types::Proposal>,
    #[prost(message, optional, tag = "2")]
    pub error: ::core::option::Option<RemoteSignerError>,
}
impl ::prost::Name for SignedProposalResponse {
    const NAME: &'static str = "SignedProposalResponse";
    const PACKAGE: &'static str = "tendermint.privval";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.privval.{}", Self::NAME)
    }
}
/// PingRequest is a request to confirm that the connection is alive.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PingRequest {}
impl ::prost::Name for PingRequest {
    const NAME: &'static str = "PingRequest";
    const PACKAGE: &'static str = "tendermint.privval";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.privval.{}", Self::NAME)
    }
}
/// PingResponse is a response to confirm that the connection is alive.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PingResponse {}
impl ::prost::Name for PingResponse {
    const NAME: &'static str = "PingResponse";
    const PACKAGE: &'static str = "tendermint.privval";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.privval.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Message {
    #[prost(oneof = "message::Sum", tags = "1, 2, 3, 4, 5, 6, 7, 8")]
    pub sum: ::core::option::Option<message::Sum>,
}
/// Nested message and enum types in `Message`.
pub mod message {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(message, tag = "1")]
        PubKeyRequest(super::PubKeyRequest),
        #[prost(message, tag = "2")]
        PubKeyResponse(super::PubKeyResponse),
        #[prost(message, tag = "3")]
        SignVoteRequest(super::SignVoteRequest),
        #[prost(message, tag = "4")]
        SignedVoteResponse(super::SignedVoteResponse),
        #[prost(message, tag = "5")]
        SignProposalRequest(super::SignProposalRequest),
        #[prost(message, tag = "6")]
        SignedProposalResponse(super::SignedProposalResponse),
        #[prost(message, tag = "7")]
        PingRequest(super::PingRequest),
        #[prost(message, tag = "8")]
        PingResponse(super::PingResponse),
    }
}
impl ::prost::Name for Message {
    const NAME: &'static str = "Message";
    const PACKAGE: &'static str = "tendermint.privval";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.privval.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
