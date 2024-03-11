// @generated
/// Module defines the ORM module which adds providers to the app container for
/// ORM ModuleDB's and in the future will automatically register query
/// services for modules that use the ORM.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Module {}
impl ::prost::Name for Module {
    const NAME: &'static str = "Module";
    const PACKAGE: &'static str = "cosmos.orm.module.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.orm.module.v1alpha1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
