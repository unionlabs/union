// @generated
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
impl ::prost::Name for Snapshot {
    const NAME: &'static str = "Snapshot";
    const PACKAGE: &'static str = "cosmos.store.snapshots.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.store.snapshots.v1.{}", Self::NAME)
    }
}
/// Metadata contains SDK-specific snapshot metadata.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Metadata {
    /// SHA-256 chunk hashes
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub chunk_hashes: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
impl ::prost::Name for Metadata {
    const NAME: &'static str = "Metadata";
    const PACKAGE: &'static str = "cosmos.store.snapshots.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.store.snapshots.v1.{}", Self::NAME)
    }
}
/// SnapshotItem is an item contained in a rootmulti.Store snapshot.
///
/// Since: cosmos-sdk 0.46
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SnapshotItem {
    /// item is the specific type of snapshot item.
    #[prost(oneof = "snapshot_item::Item", tags = "1, 2, 3, 4")]
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
    }
}
impl ::prost::Name for SnapshotItem {
    const NAME: &'static str = "SnapshotItem";
    const PACKAGE: &'static str = "cosmos.store.snapshots.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.store.snapshots.v1.{}", Self::NAME)
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
impl ::prost::Name for SnapshotStoreItem {
    const NAME: &'static str = "SnapshotStoreItem";
    const PACKAGE: &'static str = "cosmos.store.snapshots.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.store.snapshots.v1.{}", Self::NAME)
    }
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
impl ::prost::Name for SnapshotIavlItem {
    const NAME: &'static str = "SnapshotIAVLItem";
    const PACKAGE: &'static str = "cosmos.store.snapshots.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.store.snapshots.v1.{}", Self::NAME)
    }
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
impl ::prost::Name for SnapshotExtensionMeta {
    const NAME: &'static str = "SnapshotExtensionMeta";
    const PACKAGE: &'static str = "cosmos.store.snapshots.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.store.snapshots.v1.{}", Self::NAME)
    }
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
impl ::prost::Name for SnapshotExtensionPayload {
    const NAME: &'static str = "SnapshotExtensionPayload";
    const PACKAGE: &'static str = "cosmos.store.snapshots.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.store.snapshots.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
