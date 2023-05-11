/// ClientState defines a solo machine client that tracks the current consensus
/// state and if the client is frozen.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    /// latest sequence of the client state
    #[prost(uint64, tag = "1")]
    pub sequence: u64,
    /// frozen sequence of the solo machine
    #[prost(bool, tag = "2")]
    pub is_frozen: bool,
    #[prost(message, optional, tag = "3")]
    pub consensus_state: ::core::option::Option<ConsensusState>,
}
/// ConsensusState defines a solo machine consensus state. The sequence of a
/// consensus state is contained in the "height" key used in storing the
/// consensus state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusState {
    /// public key of the solo machine
    #[prost(message, optional, tag = "1")]
    pub public_key: ::core::option::Option<
        super::super::super::super::google::protobuf::Any,
    >,
    /// diversifier allows the same public key to be re-used across different solo
    /// machine clients (potentially on different chains) without being considered
    /// misbehaviour.
    #[prost(string, tag = "2")]
    pub diversifier: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub timestamp: u64,
}
/// Header defines a solo machine consensus header
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
    #[prost(uint64, tag = "1")]
    pub timestamp: u64,
    #[prost(bytes = "vec", tag = "2")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub new_public_key: ::core::option::Option<
        super::super::super::super::google::protobuf::Any,
    >,
    #[prost(string, tag = "4")]
    pub new_diversifier: ::prost::alloc::string::String,
}
/// Misbehaviour defines misbehaviour for a solo machine which consists
/// of a sequence and two signatures over different messages at that sequence.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Misbehaviour {
    #[prost(uint64, tag = "1")]
    pub sequence: u64,
    #[prost(message, optional, tag = "2")]
    pub signature_one: ::core::option::Option<SignatureAndData>,
    #[prost(message, optional, tag = "3")]
    pub signature_two: ::core::option::Option<SignatureAndData>,
}
/// SignatureAndData contains a signature and the data signed over to create that
/// signature.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignatureAndData {
    #[prost(bytes = "vec", tag = "1")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub path: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "4")]
    pub timestamp: u64,
}
/// TimestampedSignatureData contains the signature data and the timestamp of the
/// signature.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TimestampedSignatureData {
    #[prost(bytes = "vec", tag = "1")]
    pub signature_data: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub timestamp: u64,
}
/// SignBytes defines the signed bytes used for signature verification.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignBytes {
    /// the sequence number
    #[prost(uint64, tag = "1")]
    pub sequence: u64,
    /// the proof timestamp
    #[prost(uint64, tag = "2")]
    pub timestamp: u64,
    /// the public key diversifier
    #[prost(string, tag = "3")]
    pub diversifier: ::prost::alloc::string::String,
    /// the standardised path bytes
    #[prost(bytes = "vec", tag = "4")]
    pub path: ::prost::alloc::vec::Vec<u8>,
    /// the marshaled data bytes
    #[prost(bytes = "vec", tag = "5")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
/// HeaderData returns the SignBytes data for update verification.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HeaderData {
    /// header public key
    #[prost(message, optional, tag = "1")]
    pub new_pub_key: ::core::option::Option<
        super::super::super::super::google::protobuf::Any,
    >,
    /// header diversifier
    #[prost(string, tag = "2")]
    pub new_diversifier: ::prost::alloc::string::String,
}
