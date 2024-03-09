// @generated
/// GenesisState defines the ibc module's genesis state.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// ICS002 - Clients genesis state
    #[prost(message, optional, tag = "1")]
    pub client_genesis: ::core::option::Option<super::super::client::v1::GenesisState>,
    /// ICS003 - Connections genesis state
    #[prost(message, optional, tag = "2")]
    pub connection_genesis: ::core::option::Option<super::super::connection::v1::GenesisState>,
    /// ICS004 - Channel genesis state
    #[prost(message, optional, tag = "3")]
    pub channel_genesis: ::core::option::Option<super::super::channel::v1::GenesisState>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "ibc.core.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.types.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
