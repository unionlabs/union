// @generated
/// Module is the config object of the bank module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Module {
    /// blocked_module_accounts_override configures exceptional module accounts which should be blocked from receiving
    /// funds. If left empty it defaults to the list of account names supplied in the auth module configuration as
    /// module_account_permissions
    #[prost(string, repeated, tag = "1")]
    pub blocked_module_accounts_override: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// authority defines the custom module authority. If not set, defaults to the governance module.
    #[prost(string, tag = "2")]
    pub authority: ::prost::alloc::string::String,
    /// restrictions_order specifies the order of send restrictions and should be
    /// a list of module names which provide a send restriction instance. If no
    /// order is provided, then restrictions will be applied in alphabetical order
    /// of module names.
    #[prost(string, repeated, tag = "3")]
    pub restrictions_order: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for Module {
    const NAME: &'static str = "Module";
    const PACKAGE: &'static str = "cosmos.bank.module.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.bank.module.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
