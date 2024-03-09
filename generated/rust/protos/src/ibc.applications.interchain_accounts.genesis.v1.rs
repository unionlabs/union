// @generated
/// GenesisState defines the interchain accounts genesis state
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    #[prost(message, optional, tag = "1")]
    pub controller_genesis_state: ::core::option::Option<ControllerGenesisState>,
    #[prost(message, optional, tag = "2")]
    pub host_genesis_state: ::core::option::Option<HostGenesisState>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "ibc.applications.interchain_accounts.genesis.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "ibc.applications.interchain_accounts.genesis.v1.{}",
            Self::NAME
        )
    }
}
/// ControllerGenesisState defines the interchain accounts controller genesis state
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ControllerGenesisState {
    #[prost(message, repeated, tag = "1")]
    pub active_channels: ::prost::alloc::vec::Vec<ActiveChannel>,
    #[prost(message, repeated, tag = "2")]
    pub interchain_accounts: ::prost::alloc::vec::Vec<RegisteredInterchainAccount>,
    #[prost(string, repeated, tag = "3")]
    pub ports: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, optional, tag = "4")]
    pub params: ::core::option::Option<super::super::controller::v1::Params>,
}
impl ::prost::Name for ControllerGenesisState {
    const NAME: &'static str = "ControllerGenesisState";
    const PACKAGE: &'static str = "ibc.applications.interchain_accounts.genesis.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "ibc.applications.interchain_accounts.genesis.v1.{}",
            Self::NAME
        )
    }
}
/// HostGenesisState defines the interchain accounts host genesis state
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HostGenesisState {
    #[prost(message, repeated, tag = "1")]
    pub active_channels: ::prost::alloc::vec::Vec<ActiveChannel>,
    #[prost(message, repeated, tag = "2")]
    pub interchain_accounts: ::prost::alloc::vec::Vec<RegisteredInterchainAccount>,
    #[prost(string, tag = "3")]
    pub port: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "4")]
    pub params: ::core::option::Option<super::super::host::v1::Params>,
}
impl ::prost::Name for HostGenesisState {
    const NAME: &'static str = "HostGenesisState";
    const PACKAGE: &'static str = "ibc.applications.interchain_accounts.genesis.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "ibc.applications.interchain_accounts.genesis.v1.{}",
            Self::NAME
        )
    }
}
/// ActiveChannel contains a connection ID, port ID and associated active channel ID, as well as a boolean flag to
/// indicate if the channel is middleware enabled
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ActiveChannel {
    #[prost(string, tag = "1")]
    pub connection_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub port_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub channel_id: ::prost::alloc::string::String,
    #[prost(bool, tag = "4")]
    pub is_middleware_enabled: bool,
}
impl ::prost::Name for ActiveChannel {
    const NAME: &'static str = "ActiveChannel";
    const PACKAGE: &'static str = "ibc.applications.interchain_accounts.genesis.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "ibc.applications.interchain_accounts.genesis.v1.{}",
            Self::NAME
        )
    }
}
/// RegisteredInterchainAccount contains a connection ID, port ID and associated interchain account address
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RegisteredInterchainAccount {
    #[prost(string, tag = "1")]
    pub connection_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub port_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub account_address: ::prost::alloc::string::String,
}
impl ::prost::Name for RegisteredInterchainAccount {
    const NAME: &'static str = "RegisteredInterchainAccount";
    const PACKAGE: &'static str = "ibc.applications.interchain_accounts.genesis.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "ibc.applications.interchain_accounts.genesis.v1.{}",
            Self::NAME
        )
    }
}
// @@protoc_insertion_point(module)
