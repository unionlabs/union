// @generated
/// ConnectionEnd defines a stateful object on a chain connected to another
/// separate one.
/// NOTE: there must only be 2 defined ConnectionEnds to establish
/// a connection between two chains.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConnectionEnd {
    /// client associated with this connection.
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    /// IBC version which can be utilised to determine encodings or protocols for
    /// channels or packets utilising this connection.
    #[prost(message, repeated, tag = "2")]
    pub versions: ::prost::alloc::vec::Vec<Version>,
    /// current state of the connection end.
    #[prost(enumeration = "State", tag = "3")]
    pub state: i32,
    /// counterparty chain associated with this connection.
    #[prost(message, optional, tag = "4")]
    pub counterparty: ::core::option::Option<Counterparty>,
    /// delay period that must pass before a consensus state can be used for
    /// packet-verification NOTE: delay period logic is only implemented by some
    /// clients.
    #[prost(uint64, tag = "5")]
    pub delay_period: u64,
}
impl ::prost::Name for ConnectionEnd {
    const NAME: &'static str = "ConnectionEnd";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// IdentifiedConnection defines a connection with additional connection
/// identifier field.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IdentifiedConnection {
    /// connection identifier.
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    /// client associated with this connection.
    #[prost(string, tag = "2")]
    pub client_id: ::prost::alloc::string::String,
    /// IBC version which can be utilised to determine encodings or protocols for
    /// channels or packets utilising this connection
    #[prost(message, repeated, tag = "3")]
    pub versions: ::prost::alloc::vec::Vec<Version>,
    /// current state of the connection end.
    #[prost(enumeration = "State", tag = "4")]
    pub state: i32,
    /// counterparty chain associated with this connection.
    #[prost(message, optional, tag = "5")]
    pub counterparty: ::core::option::Option<Counterparty>,
    /// delay period associated with this connection.
    #[prost(uint64, tag = "6")]
    pub delay_period: u64,
}
impl ::prost::Name for IdentifiedConnection {
    const NAME: &'static str = "IdentifiedConnection";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// Counterparty defines the counterparty chain associated with a connection end.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Counterparty {
    /// identifies the client on the counterparty chain associated with a given
    /// connection.
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    /// identifies the connection end on the counterparty chain associated with a
    /// given connection.
    #[prost(string, tag = "2")]
    pub connection_id: ::prost::alloc::string::String,
    /// commitment merkle prefix of the counterparty chain.
    #[prost(message, optional, tag = "3")]
    pub prefix: ::core::option::Option<super::super::commitment::v1::MerklePrefix>,
}
impl ::prost::Name for Counterparty {
    const NAME: &'static str = "Counterparty";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// ClientPaths define all the connection paths for a client state.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientPaths {
    /// list of connection paths
    #[prost(string, repeated, tag = "1")]
    pub paths: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for ClientPaths {
    const NAME: &'static str = "ClientPaths";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// ConnectionPaths define all the connection paths for a given client state.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConnectionPaths {
    /// client state unique identifier
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    /// list of connection paths
    #[prost(string, repeated, tag = "2")]
    pub paths: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for ConnectionPaths {
    const NAME: &'static str = "ConnectionPaths";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// Version defines the versioning scheme used to negotiate the IBC verison in
/// the connection handshake.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Version {
    /// unique version identifier
    #[prost(string, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    /// list of features compatible with the specified identifier
    #[prost(string, repeated, tag = "2")]
    pub features: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for Version {
    const NAME: &'static str = "Version";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// Params defines the set of Connection parameters.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// maximum expected time per block (in nanoseconds), used to enforce block delay. This parameter should reflect the
    /// largest amount of time that the chain might reasonably take to produce the next block under normal operating
    /// conditions. A safe choice is 3-5x the expected time per block.
    #[prost(uint64, tag = "1")]
    pub max_expected_time_per_block: u64,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// State defines if a connection is in one of the following states:
/// INIT, TRYOPEN, OPEN or UNINITIALIZED.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum State {
    /// Default State
    UninitializedUnspecified = 0,
    /// A connection end has just started the opening handshake.
    Init = 1,
    /// A connection end has acknowledged the handshake step on the counterparty
    /// chain.
    Tryopen = 2,
    /// A connection end has completed the handshake.
    Open = 3,
}
impl State {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            State::UninitializedUnspecified => "STATE_UNINITIALIZED_UNSPECIFIED",
            State::Init => "STATE_INIT",
            State::Tryopen => "STATE_TRYOPEN",
            State::Open => "STATE_OPEN",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "STATE_UNINITIALIZED_UNSPECIFIED" => Some(Self::UninitializedUnspecified),
            "STATE_INIT" => Some(Self::Init),
            "STATE_TRYOPEN" => Some(Self::Tryopen),
            "STATE_OPEN" => Some(Self::Open),
            _ => None,
        }
    }
}
/// GenesisState defines the ibc connection submodule's genesis state.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    #[prost(message, repeated, tag = "1")]
    pub connections: ::prost::alloc::vec::Vec<IdentifiedConnection>,
    #[prost(message, repeated, tag = "2")]
    pub client_connection_paths: ::prost::alloc::vec::Vec<ConnectionPaths>,
    /// the sequence for the next generated connection identifier
    #[prost(uint64, tag = "3")]
    pub next_connection_sequence: u64,
    #[prost(message, optional, tag = "4")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// QueryConnectionRequest is the request type for the Query/Connection RPC
/// method
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConnectionRequest {
    /// connection unique identifier
    #[prost(string, tag = "1")]
    pub connection_id: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryConnectionRequest {
    const NAME: &'static str = "QueryConnectionRequest";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// QueryConnectionResponse is the response type for the Query/Connection RPC
/// method. Besides the connection end, it includes a proof and the height from
/// which the proof was retrieved.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConnectionResponse {
    /// connection associated with the request identifier
    #[prost(message, optional, tag = "1")]
    pub connection: ::core::option::Option<ConnectionEnd>,
    /// merkle proof of existence
    #[prost(bytes = "vec", tag = "2")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
    /// height at which the proof was retrieved
    #[prost(message, optional, tag = "3")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
}
impl ::prost::Name for QueryConnectionResponse {
    const NAME: &'static str = "QueryConnectionResponse";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// QueryConnectionsRequest is the request type for the Query/Connections RPC
/// method
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConnectionsRequest {
    #[prost(message, optional, tag = "1")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageRequest,
    >,
}
impl ::prost::Name for QueryConnectionsRequest {
    const NAME: &'static str = "QueryConnectionsRequest";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// QueryConnectionsResponse is the response type for the Query/Connections RPC
/// method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConnectionsResponse {
    /// list of stored connections of the chain.
    #[prost(message, repeated, tag = "1")]
    pub connections: ::prost::alloc::vec::Vec<IdentifiedConnection>,
    /// pagination response
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageResponse,
    >,
    /// query block height
    #[prost(message, optional, tag = "3")]
    pub height: ::core::option::Option<super::super::client::v1::Height>,
}
impl ::prost::Name for QueryConnectionsResponse {
    const NAME: &'static str = "QueryConnectionsResponse";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// QueryClientConnectionsRequest is the request type for the
/// Query/ClientConnections RPC method
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryClientConnectionsRequest {
    /// client identifier associated with a connection
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryClientConnectionsRequest {
    const NAME: &'static str = "QueryClientConnectionsRequest";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// QueryClientConnectionsResponse is the response type for the
/// Query/ClientConnections RPC method
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryClientConnectionsResponse {
    /// slice of all the connection paths associated with a client.
    #[prost(string, repeated, tag = "1")]
    pub connection_paths: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// merkle proof of existence
    #[prost(bytes = "vec", tag = "2")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
    /// height at which the proof was generated
    #[prost(message, optional, tag = "3")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
}
impl ::prost::Name for QueryClientConnectionsResponse {
    const NAME: &'static str = "QueryClientConnectionsResponse";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// QueryConnectionClientStateRequest is the request type for the
/// Query/ConnectionClientState RPC method
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConnectionClientStateRequest {
    /// connection identifier
    #[prost(string, tag = "1")]
    pub connection_id: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryConnectionClientStateRequest {
    const NAME: &'static str = "QueryConnectionClientStateRequest";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// QueryConnectionClientStateResponse is the response type for the
/// Query/ConnectionClientState RPC method
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConnectionClientStateResponse {
    /// client state associated with the channel
    #[prost(message, optional, tag = "1")]
    pub identified_client_state:
        ::core::option::Option<super::super::client::v1::IdentifiedClientState>,
    /// merkle proof of existence
    #[prost(bytes = "vec", tag = "2")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
    /// height at which the proof was retrieved
    #[prost(message, optional, tag = "3")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
}
impl ::prost::Name for QueryConnectionClientStateResponse {
    const NAME: &'static str = "QueryConnectionClientStateResponse";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// QueryConnectionConsensusStateRequest is the request type for the
/// Query/ConnectionConsensusState RPC method
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConnectionConsensusStateRequest {
    /// connection identifier
    #[prost(string, tag = "1")]
    pub connection_id: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub revision_number: u64,
    #[prost(uint64, tag = "3")]
    pub revision_height: u64,
}
impl ::prost::Name for QueryConnectionConsensusStateRequest {
    const NAME: &'static str = "QueryConnectionConsensusStateRequest";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// QueryConnectionConsensusStateResponse is the response type for the
/// Query/ConnectionConsensusState RPC method
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConnectionConsensusStateResponse {
    /// consensus state associated with the channel
    #[prost(message, optional, tag = "1")]
    pub consensus_state: ::core::option::Option<super::super::super::super::google::protobuf::Any>,
    /// client ID associated with the consensus state
    #[prost(string, tag = "2")]
    pub client_id: ::prost::alloc::string::String,
    /// merkle proof of existence
    #[prost(bytes = "vec", tag = "3")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
    /// height at which the proof was retrieved
    #[prost(message, optional, tag = "4")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
}
impl ::prost::Name for QueryConnectionConsensusStateResponse {
    const NAME: &'static str = "QueryConnectionConsensusStateResponse";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// QueryConnectionParamsRequest is the request type for the Query/ConnectionParams RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConnectionParamsRequest {}
impl ::prost::Name for QueryConnectionParamsRequest {
    const NAME: &'static str = "QueryConnectionParamsRequest";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// QueryConnectionParamsResponse is the response type for the Query/ConnectionParams RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConnectionParamsResponse {
    /// params defines the parameters of the module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for QueryConnectionParamsResponse {
    const NAME: &'static str = "QueryConnectionParamsResponse";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// MsgConnectionOpenInit defines the msg sent by an account on Chain A to
/// initialize a connection with Chain B.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgConnectionOpenInit {
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub counterparty: ::core::option::Option<Counterparty>,
    #[prost(message, optional, tag = "3")]
    pub version: ::core::option::Option<Version>,
    #[prost(uint64, tag = "4")]
    pub delay_period: u64,
    #[prost(string, tag = "5")]
    pub signer: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgConnectionOpenInit {
    const NAME: &'static str = "MsgConnectionOpenInit";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// MsgConnectionOpenInitResponse defines the Msg/ConnectionOpenInit response
/// type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgConnectionOpenInitResponse {}
impl ::prost::Name for MsgConnectionOpenInitResponse {
    const NAME: &'static str = "MsgConnectionOpenInitResponse";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// MsgConnectionOpenTry defines a msg sent by a Relayer to try to open a
/// connection on Chain B.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgConnectionOpenTry {
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    /// Deprecated: this field is unused. Crossing hellos are no longer supported in core IBC.
    #[deprecated]
    #[prost(string, tag = "2")]
    pub previous_connection_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub client_state: ::core::option::Option<super::super::super::super::google::protobuf::Any>,
    #[prost(message, optional, tag = "4")]
    pub counterparty: ::core::option::Option<Counterparty>,
    #[prost(uint64, tag = "5")]
    pub delay_period: u64,
    #[prost(message, repeated, tag = "6")]
    pub counterparty_versions: ::prost::alloc::vec::Vec<Version>,
    #[prost(message, optional, tag = "7")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
    /// proof of the initialization the connection on Chain A: `UNITIALIZED ->
    /// INIT`
    #[prost(bytes = "vec", tag = "8")]
    pub proof_init: ::prost::alloc::vec::Vec<u8>,
    /// proof of client state included in message
    #[prost(bytes = "vec", tag = "9")]
    pub proof_client: ::prost::alloc::vec::Vec<u8>,
    /// proof of client consensus state
    #[prost(bytes = "vec", tag = "10")]
    pub proof_consensus: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "11")]
    pub consensus_height: ::core::option::Option<super::super::client::v1::Height>,
    #[prost(string, tag = "12")]
    pub signer: ::prost::alloc::string::String,
    /// optional proof data for host state machines that are unable to introspect their own consensus state
    #[prost(bytes = "vec", tag = "13")]
    pub host_consensus_state_proof: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgConnectionOpenTry {
    const NAME: &'static str = "MsgConnectionOpenTry";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// MsgConnectionOpenTryResponse defines the Msg/ConnectionOpenTry response type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgConnectionOpenTryResponse {}
impl ::prost::Name for MsgConnectionOpenTryResponse {
    const NAME: &'static str = "MsgConnectionOpenTryResponse";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// MsgConnectionOpenAck defines a msg sent by a Relayer to Chain A to
/// acknowledge the change of connection state to TRYOPEN on Chain B.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgConnectionOpenAck {
    #[prost(string, tag = "1")]
    pub connection_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub counterparty_connection_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub version: ::core::option::Option<Version>,
    #[prost(message, optional, tag = "4")]
    pub client_state: ::core::option::Option<super::super::super::super::google::protobuf::Any>,
    #[prost(message, optional, tag = "5")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
    /// proof of the initialization the connection on Chain B: `UNITIALIZED ->
    /// TRYOPEN`
    #[prost(bytes = "vec", tag = "6")]
    pub proof_try: ::prost::alloc::vec::Vec<u8>,
    /// proof of client state included in message
    #[prost(bytes = "vec", tag = "7")]
    pub proof_client: ::prost::alloc::vec::Vec<u8>,
    /// proof of client consensus state
    #[prost(bytes = "vec", tag = "8")]
    pub proof_consensus: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "9")]
    pub consensus_height: ::core::option::Option<super::super::client::v1::Height>,
    #[prost(string, tag = "10")]
    pub signer: ::prost::alloc::string::String,
    /// optional proof data for host state machines that are unable to introspect their own consensus state
    #[prost(bytes = "vec", tag = "11")]
    pub host_consensus_state_proof: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgConnectionOpenAck {
    const NAME: &'static str = "MsgConnectionOpenAck";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// MsgConnectionOpenAckResponse defines the Msg/ConnectionOpenAck response type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgConnectionOpenAckResponse {}
impl ::prost::Name for MsgConnectionOpenAckResponse {
    const NAME: &'static str = "MsgConnectionOpenAckResponse";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// MsgConnectionOpenConfirm defines a msg sent by a Relayer to Chain B to
/// acknowledge the change of connection state to OPEN on Chain A.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgConnectionOpenConfirm {
    #[prost(string, tag = "1")]
    pub connection_id: ::prost::alloc::string::String,
    /// proof for the change of the connection state on Chain A: `INIT -> OPEN`
    #[prost(bytes = "vec", tag = "2")]
    pub proof_ack: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub proof_height: ::core::option::Option<super::super::client::v1::Height>,
    #[prost(string, tag = "4")]
    pub signer: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgConnectionOpenConfirm {
    const NAME: &'static str = "MsgConnectionOpenConfirm";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// MsgConnectionOpenConfirmResponse defines the Msg/ConnectionOpenConfirm
/// response type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgConnectionOpenConfirmResponse {}
impl ::prost::Name for MsgConnectionOpenConfirmResponse {
    const NAME: &'static str = "MsgConnectionOpenConfirmResponse";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// MsgUpdateParams defines the sdk.Msg type to update the connection parameters.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParams {
    /// signer address
    #[prost(string, tag = "1")]
    pub signer: ::prost::alloc::string::String,
    /// params defines the connection parameters to update.
    ///
    /// NOTE: All parameters must be supplied.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for MsgUpdateParams {
    const NAME: &'static str = "MsgUpdateParams";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
/// MsgUpdateParamsResponse defines the MsgUpdateParams response type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParamsResponse {}
impl ::prost::Name for MsgUpdateParamsResponse {
    const NAME: &'static str = "MsgUpdateParamsResponse";
    const PACKAGE: &'static str = "ibc.core.connection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.connection.v1.{}", Self::NAME)
    }
}
include!("ibc.core.connection.v1.tonic.rs");
// @@protoc_insertion_point(module)
