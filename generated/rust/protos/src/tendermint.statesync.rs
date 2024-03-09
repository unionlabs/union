// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Message {
    #[prost(oneof = "message::Sum", tags = "1, 2, 3, 4")]
    pub sum: ::core::option::Option<message::Sum>,
}
/// Nested message and enum types in `Message`.
pub mod message {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(message, tag = "1")]
        SnapshotsRequest(super::SnapshotsRequest),
        #[prost(message, tag = "2")]
        SnapshotsResponse(super::SnapshotsResponse),
        #[prost(message, tag = "3")]
        ChunkRequest(super::ChunkRequest),
        #[prost(message, tag = "4")]
        ChunkResponse(super::ChunkResponse),
    }
}
impl ::prost::Name for Message {
    const NAME: &'static str = "Message";
    const PACKAGE: &'static str = "tendermint.statesync";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.statesync.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SnapshotsRequest {}
impl ::prost::Name for SnapshotsRequest {
    const NAME: &'static str = "SnapshotsRequest";
    const PACKAGE: &'static str = "tendermint.statesync";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.statesync.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SnapshotsResponse {
    #[prost(uint64, tag = "1")]
    pub height: u64,
    #[prost(uint32, tag = "2")]
    pub format: u32,
    #[prost(uint32, tag = "3")]
    pub chunks: u32,
    #[prost(bytes = "vec", tag = "4")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "5")]
    pub metadata: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for SnapshotsResponse {
    const NAME: &'static str = "SnapshotsResponse";
    const PACKAGE: &'static str = "tendermint.statesync";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.statesync.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChunkRequest {
    #[prost(uint64, tag = "1")]
    pub height: u64,
    #[prost(uint32, tag = "2")]
    pub format: u32,
    #[prost(uint32, tag = "3")]
    pub index: u32,
}
impl ::prost::Name for ChunkRequest {
    const NAME: &'static str = "ChunkRequest";
    const PACKAGE: &'static str = "tendermint.statesync";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.statesync.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChunkResponse {
    #[prost(uint64, tag = "1")]
    pub height: u64,
    #[prost(uint32, tag = "2")]
    pub format: u32,
    #[prost(uint32, tag = "3")]
    pub index: u32,
    #[prost(bytes = "vec", tag = "4")]
    pub chunk: ::prost::alloc::vec::Vec<u8>,
    #[prost(bool, tag = "5")]
    pub missing: bool,
}
impl ::prost::Name for ChunkResponse {
    const NAME: &'static str = "ChunkResponse";
    const PACKAGE: &'static str = "tendermint.statesync";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.statesync.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
