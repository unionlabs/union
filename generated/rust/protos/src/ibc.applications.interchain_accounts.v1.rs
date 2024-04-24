// @generated
/// InterchainAccountPacketData is comprised of a raw transaction, type of transaction and optional memo field.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InterchainAccountPacketData {
    #[prost(enumeration = "Type", tag = "1")]
    pub r#type: i32,
    #[prost(bytes = "vec", tag = "2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "3")]
    pub memo: ::prost::alloc::string::String,
}
impl ::prost::Name for InterchainAccountPacketData {
    const NAME: &'static str = "InterchainAccountPacketData";
    const PACKAGE: &'static str = "ibc.applications.interchain_accounts.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.interchain_accounts.v1.{}", Self::NAME)
    }
}
/// CosmosTx contains a list of sdk.Msg's. It should be used when sending transactions to an SDK host chain.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CosmosTx {
    #[prost(message, repeated, tag = "1")]
    pub messages: ::prost::alloc::vec::Vec<super::super::super::super::google::protobuf::Any>,
}
impl ::prost::Name for CosmosTx {
    const NAME: &'static str = "CosmosTx";
    const PACKAGE: &'static str = "ibc.applications.interchain_accounts.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.interchain_accounts.v1.{}", Self::NAME)
    }
}
/// Type defines a classification of message issued from a controller chain to its associated interchain accounts
/// host
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Type {
    /// Default zero value enumeration
    Unspecified = 0,
    /// Execute a transaction on an interchain accounts host chain
    ExecuteTx = 1,
}
impl Type {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Type::Unspecified => "TYPE_UNSPECIFIED",
            Type::ExecuteTx => "TYPE_EXECUTE_TX",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "TYPE_EXECUTE_TX" => Some(Self::ExecuteTx),
            _ => None,
        }
    }
}
/// An InterchainAccount is defined as a BaseAccount & the address of the account owner on the controller chain
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InterchainAccount {
    #[prost(message, optional, tag = "1")]
    pub base_account:
        ::core::option::Option<super::super::super::super::cosmos::auth::v1beta1::BaseAccount>,
    #[prost(string, tag = "2")]
    pub account_owner: ::prost::alloc::string::String,
}
impl ::prost::Name for InterchainAccount {
    const NAME: &'static str = "InterchainAccount";
    const PACKAGE: &'static str = "ibc.applications.interchain_accounts.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.interchain_accounts.v1.{}", Self::NAME)
    }
}
/// Metadata defines a set of protocol specific data encoded into the ICS27 channel version bytestring
/// See ICS004: <https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#Versioning>
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Metadata {
    /// version defines the ICS27 protocol version
    #[prost(string, tag = "1")]
    pub version: ::prost::alloc::string::String,
    /// controller_connection_id is the connection identifier associated with the controller chain
    #[prost(string, tag = "2")]
    pub controller_connection_id: ::prost::alloc::string::String,
    /// host_connection_id is the connection identifier associated with the host chain
    #[prost(string, tag = "3")]
    pub host_connection_id: ::prost::alloc::string::String,
    /// address defines the interchain account address to be fulfilled upon the OnChanOpenTry handshake step
    /// NOTE: the address field is empty on the OnChanOpenInit handshake step
    #[prost(string, tag = "4")]
    pub address: ::prost::alloc::string::String,
    /// encoding defines the supported codec format
    #[prost(string, tag = "5")]
    pub encoding: ::prost::alloc::string::String,
    /// tx_type defines the type of transactions the interchain account can execute
    #[prost(string, tag = "6")]
    pub tx_type: ::prost::alloc::string::String,
}
impl ::prost::Name for Metadata {
    const NAME: &'static str = "Metadata";
    const PACKAGE: &'static str = "ibc.applications.interchain_accounts.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.interchain_accounts.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
