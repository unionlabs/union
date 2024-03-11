// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateUnionValidator {
    #[prost(message, optional, tag = "1")]
    pub underlying:
        ::core::option::Option<super::super::super::cosmos::staking::v1beta1::MsgCreateValidator>,
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "3")]
    pub proof_of_possession: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgCreateUnionValidator {
    const NAME: &'static str = "MsgCreateUnionValidator";
    const PACKAGE: &'static str = "union.staking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.staking.v1.{}", Self::NAME)
    }
}
include!("union.staking.v1.tonic.rs");
// @@protoc_insertion_point(module)
