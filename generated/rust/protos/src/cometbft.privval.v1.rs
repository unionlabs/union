// @generated
/// remotesignererror is returned when the remote signer fails.
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
    const PACKAGE: &'static str = "cometbft.privval.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.privval.v1.{}", Self::NAME)
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
    const PACKAGE: &'static str = "cometbft.privval.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.privval.v1.{}", Self::NAME)
    }
}
/// PubKeyResponse is a response message containing the public key.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PubKeyResponse {
    #[prost(message, optional, tag = "2")]
    pub error: ::core::option::Option<RemoteSignerError>,
    #[prost(bytes = "vec", tag = "3")]
    pub pub_key_bytes: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "4")]
    pub pub_key_type: ::prost::alloc::string::String,
}
impl ::prost::Name for PubKeyResponse {
    const NAME: &'static str = "PubKeyResponse";
    const PACKAGE: &'static str = "cometbft.privval.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.privval.v1.{}", Self::NAME)
    }
}
/// SignVoteRequest is a request to sign a vote
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignVoteRequest {
    #[prost(message, optional, tag = "1")]
    pub vote: ::core::option::Option<super::super::types::v1::Vote>,
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
    /// if true, the signer may skip signing the extension bytes.
    #[prost(bool, tag = "3")]
    pub skip_extension_signing: bool,
}
impl ::prost::Name for SignVoteRequest {
    const NAME: &'static str = "SignVoteRequest";
    const PACKAGE: &'static str = "cometbft.privval.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.privval.v1.{}", Self::NAME)
    }
}
/// SignedVoteResponse is a response containing a signed vote or an error
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignedVoteResponse {
    #[prost(message, optional, tag = "1")]
    pub vote: ::core::option::Option<super::super::types::v1::Vote>,
    #[prost(message, optional, tag = "2")]
    pub error: ::core::option::Option<RemoteSignerError>,
}
impl ::prost::Name for SignedVoteResponse {
    const NAME: &'static str = "SignedVoteResponse";
    const PACKAGE: &'static str = "cometbft.privval.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.privval.v1.{}", Self::NAME)
    }
}
/// SignProposalRequest is a request to sign a proposal
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignProposalRequest {
    #[prost(message, optional, tag = "1")]
    pub proposal: ::core::option::Option<super::super::types::v1::Proposal>,
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
}
impl ::prost::Name for SignProposalRequest {
    const NAME: &'static str = "SignProposalRequest";
    const PACKAGE: &'static str = "cometbft.privval.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.privval.v1.{}", Self::NAME)
    }
}
/// SignedProposalResponse is response containing a signed proposal or an error
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignedProposalResponse {
    #[prost(message, optional, tag = "1")]
    pub proposal: ::core::option::Option<super::super::types::v1::Proposal>,
    #[prost(message, optional, tag = "2")]
    pub error: ::core::option::Option<RemoteSignerError>,
}
impl ::prost::Name for SignedProposalResponse {
    const NAME: &'static str = "SignedProposalResponse";
    const PACKAGE: &'static str = "cometbft.privval.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.privval.v1.{}", Self::NAME)
    }
}
/// SignBytesRequest is a request to sign arbitrary bytes
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignBytesRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for SignBytesRequest {
    const NAME: &'static str = "SignBytesRequest";
    const PACKAGE: &'static str = "cometbft.privval.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.privval.v1.{}", Self::NAME)
    }
}
/// SignBytesResponse is a response containing a signature or an error
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignBytesResponse {
    #[prost(bytes = "vec", tag = "1")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub error: ::core::option::Option<RemoteSignerError>,
}
impl ::prost::Name for SignBytesResponse {
    const NAME: &'static str = "SignBytesResponse";
    const PACKAGE: &'static str = "cometbft.privval.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.privval.v1.{}", Self::NAME)
    }
}
/// PingRequest is a request to confirm that the connection is alive.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PingRequest {}
impl ::prost::Name for PingRequest {
    const NAME: &'static str = "PingRequest";
    const PACKAGE: &'static str = "cometbft.privval.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.privval.v1.{}", Self::NAME)
    }
}
/// PingResponse is a response to confirm that the connection is alive.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PingResponse {}
impl ::prost::Name for PingResponse {
    const NAME: &'static str = "PingResponse";
    const PACKAGE: &'static str = "cometbft.privval.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.privval.v1.{}", Self::NAME)
    }
}
/// Message is an abstract message to/from the remote signer.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Message {
    /// Sum of all possible messages.
    #[prost(oneof = "message::Sum", tags = "1, 2, 3, 4, 5, 6, 7, 8, 9, 10")]
    pub sum: ::core::option::Option<message::Sum>,
}
/// Nested message and enum types in `Message`.
pub mod message {
    /// Sum of all possible messages.
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
        #[prost(message, tag = "9")]
        SignBytesRequest(super::SignBytesRequest),
        #[prost(message, tag = "10")]
        SignBytesResponse(super::SignBytesResponse),
    }
}
impl ::prost::Name for Message {
    const NAME: &'static str = "Message";
    const PACKAGE: &'static str = "cometbft.privval.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.privval.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
