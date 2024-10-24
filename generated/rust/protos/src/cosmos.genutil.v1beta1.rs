// @generated
/// GenesisState defines the raw genesis transaction in JSON.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// gen_txs defines the genesis transactions.
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub gen_txs: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "cosmos.genutil.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.genutil.v1beta1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
