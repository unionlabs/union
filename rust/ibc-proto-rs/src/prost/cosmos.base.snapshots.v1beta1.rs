/// Snapshot contains Tendermint state sync snapshot info.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Snapshot {
    #[prost(uint64, tag = "1")]
    pub height: u64,
    #[prost(uint32, tag = "2")]
    pub format: u32,
    #[prost(uint32, tag = "3")]
    pub chunks: u32,
    #[prost(bytes = "vec", tag = "4")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "5")]
    pub metadata: ::core::option::Option<Metadata>,
}
/// Metadata contains SDK-specific snapshot metadata.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Metadata {
    /// SHA-256 chunk hashes
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub chunk_hashes: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
/// SnapshotItem is an item contained in a rootmulti.Store snapshot.
///
/// Since: cosmos-sdk 0.46
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SnapshotItem {
    /// item is the specific type of snapshot item.
    #[prost(oneof = "snapshot_item::Item", tags = "1, 2, 3, 4, 5, 6")]
    pub item: ::core::option::Option<snapshot_item::Item>,
}
/// Nested message and enum types in `SnapshotItem`.
pub mod snapshot_item {
    /// item is the specific type of snapshot item.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Item {
        #[prost(message, tag = "1")]
        Store(super::SnapshotStoreItem),
        #[prost(message, tag = "2")]
        Iavl(super::SnapshotIavlItem),
        #[prost(message, tag = "3")]
        Extension(super::SnapshotExtensionMeta),
        #[prost(message, tag = "4")]
        ExtensionPayload(super::SnapshotExtensionPayload),
        #[prost(message, tag = "5")]
        Kv(super::SnapshotKvItem),
        #[prost(message, tag = "6")]
        Schema(super::SnapshotSchema),
    }
}
/// SnapshotStoreItem contains metadata about a snapshotted store.
///
/// Since: cosmos-sdk 0.46
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SnapshotStoreItem {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
/// SnapshotIAVLItem is an exported IAVL node.
///
/// Since: cosmos-sdk 0.46
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SnapshotIavlItem {
    #[prost(bytes = "vec", tag = "1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub value: ::prost::alloc::vec::Vec<u8>,
    /// version is block height
    #[prost(int64, tag = "3")]
    pub version: i64,
    /// height is depth of the tree.
    #[prost(int32, tag = "4")]
    pub height: i32,
}
/// SnapshotExtensionMeta contains metadata about an external snapshotter.
///
/// Since: cosmos-sdk 0.46
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SnapshotExtensionMeta {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(uint32, tag = "2")]
    pub format: u32,
}
/// SnapshotExtensionPayload contains payloads of an external snapshotter.
///
/// Since: cosmos-sdk 0.46
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SnapshotExtensionPayload {
    #[prost(bytes = "vec", tag = "1")]
    pub payload: ::prost::alloc::vec::Vec<u8>,
}
/// SnapshotKVItem is an exported Key/Value Pair
///
/// Since: cosmos-sdk 0.46
/// Deprecated: This message was part of store/v2alpha1 which has been deleted from v0.47.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SnapshotKvItem {
    #[prost(bytes = "vec", tag = "1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
/// SnapshotSchema is an exported schema of smt store
///
/// Since: cosmos-sdk 0.46
/// Deprecated: This message was part of store/v2alpha1 which has been deleted from v0.47.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SnapshotSchema {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub keys: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
