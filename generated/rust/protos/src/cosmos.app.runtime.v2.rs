// @generated
/// Module is the config object for the runtime module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Module {
    /// app_name is the name of the app.
    #[prost(string, tag = "1")]
    pub app_name: ::prost::alloc::string::String,
    /// pre_blockers specifies the module names of pre blockers
    /// to call in the order in which they should be called. If this is left empty
    /// no pre blocker will be registered.
    #[prost(string, repeated, tag = "2")]
    pub pre_blockers: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// begin_blockers specifies the module names of begin blockers
    /// to call in the order in which they should be called. If this is left empty
    /// no begin blocker will be registered.
    #[prost(string, repeated, tag = "3")]
    pub begin_blockers: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// end_blockers specifies the module names of the end blockers
    /// to call in the order in which they should be called. If this is left empty
    /// no end blocker will be registered.
    #[prost(string, repeated, tag = "4")]
    pub end_blockers: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// tx_validators specifies the module names for tx validators
    /// If this is left empty, no tx validation will be registered.
    #[prost(string, repeated, tag = "5")]
    pub tx_validators: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// init_genesis specifies the module names of init genesis functions
    /// to call in the order in which they should be called. If this is left empty
    /// no init genesis function will be registered.
    #[prost(string, repeated, tag = "6")]
    pub init_genesis: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// export_genesis specifies the order in which to export module genesis data.
    /// If this is left empty, the init_genesis order will be used for export genesis
    /// if it is specified.
    #[prost(string, repeated, tag = "7")]
    pub export_genesis: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// order_migrations defines the order in which module migrations are performed.
    /// If this is left empty, it uses the default migration order (alphabetically).
    #[prost(string, repeated, tag = "8")]
    pub order_migrations: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// GasConfig is the config object for gas limits.
    #[prost(message, optional, tag = "9")]
    pub gas_config: ::core::option::Option<GasConfig>,
    /// override_store_keys is an optional list of overrides for the module store keys
    /// to be used in keeper construction.
    #[prost(message, repeated, tag = "10")]
    pub override_store_keys: ::prost::alloc::vec::Vec<StoreKeyConfig>,
    /// skip_store_keys is an optional list of store keys to skip when constructing the
    /// module's keeper. This is useful when a module does not have a store key.
    /// NOTE: the provided environment variable will have a fake store service.
    #[prost(string, repeated, tag = "11")]
    pub skip_store_keys: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for Module {
    const NAME: &'static str = "Module";
    const PACKAGE: &'static str = "cosmos.app.runtime.v2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.app.runtime.v2.{}", Self::NAME)
    }
}
/// GasConfig is the config object for gas limits.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GasConfig {
    #[prost(uint64, tag = "1")]
    pub validate_tx_gas_limit: u64,
    #[prost(uint64, tag = "2")]
    pub query_gas_limit: u64,
    #[prost(uint64, tag = "3")]
    pub simulation_gas_limit: u64,
}
impl ::prost::Name for GasConfig {
    const NAME: &'static str = "GasConfig";
    const PACKAGE: &'static str = "cosmos.app.runtime.v2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.app.runtime.v2.{}", Self::NAME)
    }
}
/// StoreKeyConfig may be supplied to override the default module store key, which
/// is the module name.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StoreKeyConfig {
    /// name of the module to override the store key of
    #[prost(string, tag = "1")]
    pub module_name: ::prost::alloc::string::String,
    /// the kv store key to use instead of the module name.
    #[prost(string, tag = "2")]
    pub kv_store_key: ::prost::alloc::string::String,
}
impl ::prost::Name for StoreKeyConfig {
    const NAME: &'static str = "StoreKeyConfig";
    const PACKAGE: &'static str = "cosmos.app.runtime.v2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.app.runtime.v2.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
