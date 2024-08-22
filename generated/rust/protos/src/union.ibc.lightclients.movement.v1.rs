// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    #[prost(bytes = "vec", tag = "1")]
    pub l1_contract_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub l2_contract_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub table_handle: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "4")]
    pub frozen_height:
        ::core::option::Option<super::super::super::super::super::ibc::core::client::v1::Height>,
}
impl ::prost::Name for ClientState {
    const NAME: &'static str = "ClientState";
    const PACKAGE: &'static str = "union.ibc.lightclients.movement.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.movement.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
