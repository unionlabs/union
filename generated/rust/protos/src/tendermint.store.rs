// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockStoreState {
    #[prost(int64, tag = "1")]
    pub base: i64,
    #[prost(int64, tag = "2")]
    pub height: i64,
}
impl ::prost::Name for BlockStoreState {
    const NAME: &'static str = "BlockStoreState";
    const PACKAGE: &'static str = "tendermint.store";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.store.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
