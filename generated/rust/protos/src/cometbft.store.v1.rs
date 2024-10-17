// @generated
/// BlockStoreState represents the state of the block store.
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
    const PACKAGE: &'static str = "cometbft.store.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.store.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
