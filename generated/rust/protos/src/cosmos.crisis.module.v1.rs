// @generated
/// Module is the config object of the crisis module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Module {
    /// fee_collector_name is the name of the FeeCollector ModuleAccount.
    #[prost(string, tag = "1")]
    pub fee_collector_name: ::prost::alloc::string::String,
    /// authority defines the custom module authority. If not set, defaults to the governance module.
    #[prost(string, tag = "2")]
    pub authority: ::prost::alloc::string::String,
}
impl ::prost::Name for Module {
    const NAME: &'static str = "Module";
    const PACKAGE: &'static str = "cosmos.crisis.module.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.crisis.module.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
