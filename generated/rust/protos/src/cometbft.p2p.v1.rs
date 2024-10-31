// @generated
/// NetAddress represents a peer's network address.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NetAddress {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub ip: ::prost::alloc::string::String,
    #[prost(uint32, tag = "3")]
    pub port: u32,
}
impl ::prost::Name for NetAddress {
    const NAME: &'static str = "NetAddress";
    const PACKAGE: &'static str = "cometbft.p2p.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.p2p.v1.{}", Self::NAME)
    }
}
/// ProtocolVersion represents the current p2p protocol version.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProtocolVersion {
    #[prost(uint64, tag = "1")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub p2p: u64,
    #[prost(uint64, tag = "2")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub block: u64,
    #[prost(uint64, tag = "3")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub app: u64,
}
impl ::prost::Name for ProtocolVersion {
    const NAME: &'static str = "ProtocolVersion";
    const PACKAGE: &'static str = "cometbft.p2p.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.p2p.v1.{}", Self::NAME)
    }
}
/// DefaultNodeInfo is a basic node's information sent to other peers during the
/// p2p handshake.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DefaultNodeInfo {
    #[prost(message, optional, tag = "1")]
    pub protocol_version: ::core::option::Option<ProtocolVersion>,
    #[prost(string, tag = "2")]
    #[serde(alias = "id")]
    pub default_node_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub listen_addr: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub network: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub version: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "6")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_upper_unprefixed"))]
    pub channels: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "7")]
    pub moniker: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "8")]
    pub other: ::core::option::Option<DefaultNodeInfoOther>,
}
impl ::prost::Name for DefaultNodeInfo {
    const NAME: &'static str = "DefaultNodeInfo";
    const PACKAGE: &'static str = "cometbft.p2p.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.p2p.v1.{}", Self::NAME)
    }
}
/// DefaultNodeInfoOther is the misc. application specific data.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DefaultNodeInfoOther {
    #[prost(string, tag = "1")]
    pub tx_index: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub rpc_address: ::prost::alloc::string::String,
}
impl ::prost::Name for DefaultNodeInfoOther {
    const NAME: &'static str = "DefaultNodeInfoOther";
    const PACKAGE: &'static str = "cometbft.p2p.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.p2p.v1.{}", Self::NAME)
    }
}
/// PexRequest is a request for peer addresses.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PexRequest {}
impl ::prost::Name for PexRequest {
    const NAME: &'static str = "PexRequest";
    const PACKAGE: &'static str = "cometbft.p2p.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.p2p.v1.{}", Self::NAME)
    }
}
/// PexAddrs is a response with peer addresses.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PexAddrs {
    #[prost(message, repeated, tag = "1")]
    pub addrs: ::prost::alloc::vec::Vec<NetAddress>,
}
impl ::prost::Name for PexAddrs {
    const NAME: &'static str = "PexAddrs";
    const PACKAGE: &'static str = "cometbft.p2p.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.p2p.v1.{}", Self::NAME)
    }
}
/// Message is an abstract PEX message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Message {
    /// Sum of all possible messages.
    #[prost(oneof = "message::Sum", tags = "1, 2")]
    pub sum: ::core::option::Option<message::Sum>,
}
/// Nested message and enum types in `Message`.
pub mod message {
    /// Sum of all possible messages.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(message, tag = "1")]
        PexRequest(super::PexRequest),
        #[prost(message, tag = "2")]
        PexAddrs(super::PexAddrs),
    }
}
impl ::prost::Name for Message {
    const NAME: &'static str = "Message";
    const PACKAGE: &'static str = "cometbft.p2p.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.p2p.v1.{}", Self::NAME)
    }
}
/// PacketPing is a request to confirm that the connection is alive.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PacketPing {}
impl ::prost::Name for PacketPing {
    const NAME: &'static str = "PacketPing";
    const PACKAGE: &'static str = "cometbft.p2p.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.p2p.v1.{}", Self::NAME)
    }
}
/// PacketPong is a response to confirm that the connection is alive.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PacketPong {}
impl ::prost::Name for PacketPong {
    const NAME: &'static str = "PacketPong";
    const PACKAGE: &'static str = "cometbft.p2p.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.p2p.v1.{}", Self::NAME)
    }
}
/// PacketMsg contains data for the specified channel ID. EOF means the message
/// is fully received.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PacketMsg {
    #[prost(int32, tag = "1")]
    pub channel_id: i32,
    #[prost(bool, tag = "2")]
    pub eof: bool,
    #[prost(bytes = "vec", tag = "3")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for PacketMsg {
    const NAME: &'static str = "PacketMsg";
    const PACKAGE: &'static str = "cometbft.p2p.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.p2p.v1.{}", Self::NAME)
    }
}
/// Packet is an abstract p2p message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Packet {
    /// Sum of all possible messages.
    #[prost(oneof = "packet::Sum", tags = "1, 2, 3")]
    pub sum: ::core::option::Option<packet::Sum>,
}
/// Nested message and enum types in `Packet`.
pub mod packet {
    /// Sum of all possible messages.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(message, tag = "1")]
        PacketPing(super::PacketPing),
        #[prost(message, tag = "2")]
        PacketPong(super::PacketPong),
        #[prost(message, tag = "3")]
        PacketMsg(super::PacketMsg),
    }
}
impl ::prost::Name for Packet {
    const NAME: &'static str = "Packet";
    const PACKAGE: &'static str = "cometbft.p2p.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.p2p.v1.{}", Self::NAME)
    }
}
/// AuthSigMessage is sent during the authentication and contains our/remote's
/// signature along with the public key.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuthSigMessage {
    #[prost(message, optional, tag = "1")]
    pub pub_key: ::core::option::Option<super::super::crypto::v1::PublicKey>,
    #[prost(bytes = "vec", tag = "2")]
    pub sig: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for AuthSigMessage {
    const NAME: &'static str = "AuthSigMessage";
    const PACKAGE: &'static str = "cometbft.p2p.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.p2p.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
