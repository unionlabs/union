// @generated
/// Module is the config object of the consensus module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Module {
    /// authority defines the custom module authority. If not set, defaults to the governance module.
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
}
impl ::prost::Name for Module {
    const NAME: &'static str = "Module";
    const PACKAGE: &'static str = "cosmos.consensus.module.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.consensus.module.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
