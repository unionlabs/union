// @generated
/// StoreKVPair is a KVStore KVPair used for listening to state changes (Sets and
/// Deletes) It optionally includes the StoreKey for the originating KVStore and
/// a Boolean flag to distinguish between Sets and Deletes
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StoreKvPair {
    /// the store key for the KVStore this pair originates from
    #[prost(string, tag = "1")]
    pub store_key: ::prost::alloc::string::String,
    /// true indicates a delete operation, false indicates a set operation
    #[prost(bool, tag = "2")]
    pub delete: bool,
    #[prost(bytes = "vec", tag = "3")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "4")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for StoreKvPair {
    const NAME: &'static str = "StoreKVPair";
    const PACKAGE: &'static str = "cosmos.store.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.store.v1beta1.{}", Self::NAME)
    }
}
/// BlockMetadata contains all the abci event data of a block
/// the file streamer dump them into files together with the state changes.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockMetadata {
    #[prost(message, optional, tag = "6")]
    pub response_commit:
        ::core::option::Option<super::super::super::cometbft::abci::v1::CommitResponse>,
    #[prost(message, optional, tag = "7")]
    pub request_finalize_block:
        ::core::option::Option<super::super::super::cometbft::abci::v1::FinalizeBlockRequest>,
    #[prost(message, optional, tag = "8")]
    pub response_finalize_block:
        ::core::option::Option<super::super::super::cometbft::abci::v1::FinalizeBlockResponse>,
}
impl ::prost::Name for BlockMetadata {
    const NAME: &'static str = "BlockMetadata";
    const PACKAGE: &'static str = "cosmos.store.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.store.v1beta1.{}", Self::NAME)
    }
}
/// CommitInfo defines commit information used by the multi-store when committing
/// a version/height.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommitInfo {
    #[prost(int64, tag = "1")]
    pub version: i64,
    #[prost(message, repeated, tag = "2")]
    pub store_infos: ::prost::alloc::vec::Vec<StoreInfo>,
    #[prost(message, optional, tag = "3")]
    pub timestamp: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for CommitInfo {
    const NAME: &'static str = "CommitInfo";
    const PACKAGE: &'static str = "cosmos.store.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.store.v1beta1.{}", Self::NAME)
    }
}
/// StoreInfo defines store-specific commit information. It contains a reference
/// between a store name and the commit ID.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StoreInfo {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub commit_id: ::core::option::Option<CommitId>,
}
impl ::prost::Name for StoreInfo {
    const NAME: &'static str = "StoreInfo";
    const PACKAGE: &'static str = "cosmos.store.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.store.v1beta1.{}", Self::NAME)
    }
}
/// CommitID defines the commitment information when a specific store is
/// committed.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommitId {
    #[prost(int64, tag = "1")]
    pub version: i64,
    #[prost(bytes = "vec", tag = "2")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for CommitId {
    const NAME: &'static str = "CommitID";
    const PACKAGE: &'static str = "cosmos.store.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.store.v1beta1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
