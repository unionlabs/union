// @generated
/// Module is the config object of the staking module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Module {
    /// hooks_order specifies the order of staking hooks and should be a list
    /// of module names which provide a staking hooks instance. If no order is
    /// provided, then hooks will be applied in alphabetical order of module names.
    #[prost(string, repeated, tag = "1")]
    pub hooks_order: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// authority defines the custom module authority. If not set, defaults to the governance module.
    #[prost(string, tag = "2")]
    pub authority: ::prost::alloc::string::String,
    /// bech32_prefix_validator is the bech32 validator prefix for the app.
    #[prost(string, tag = "3")]
    pub bech32_prefix_validator: ::prost::alloc::string::String,
    /// bech32_prefix_consensus is the bech32 consensus node prefix for the app.
    #[prost(string, tag = "4")]
    pub bech32_prefix_consensus: ::prost::alloc::string::String,
}
impl ::prost::Name for Module {
    const NAME: &'static str = "Module";
    const PACKAGE: &'static str = "cosmos.staking.module.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.staking.module.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
