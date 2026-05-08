/// Module is the config object of the staking module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct Module {}
impl ::prost::Name for Module {
    const NAME: &'static str = "Module";
    const PACKAGE: &'static str = "union.staking.module.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.staking.module.v1.{}", Self::NAME)
    }
}
