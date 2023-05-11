/// CommitInfo defines commit information used by the multi-store when committing
/// a version/height.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommitInfo {
    #[prost(int64, tag = "1")]
    pub version: i64,
    #[prost(message, repeated, tag = "2")]
    pub store_infos: ::prost::alloc::vec::Vec<StoreInfo>,
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
/// StoreKVPair is a KVStore KVPair used for listening to state changes (Sets and Deletes)
/// It optionally includes the StoreKey for the originating KVStore and a Boolean flag to distinguish between Sets and
/// Deletes
///
/// Since: cosmos-sdk 0.43
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
/// BlockMetadata contains all the abci event data of a block
/// the file streamer dump them into files together with the state changes.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockMetadata {
    #[prost(message, optional, tag = "1")]
    pub request_begin_block: ::core::option::Option<
        ::tendermint_proto::abci::RequestBeginBlock,
    >,
    #[prost(message, optional, tag = "2")]
    pub response_begin_block: ::core::option::Option<
        ::tendermint_proto::abci::ResponseBeginBlock,
    >,
    #[prost(message, repeated, tag = "3")]
    pub deliver_txs: ::prost::alloc::vec::Vec<block_metadata::DeliverTx>,
    #[prost(message, optional, tag = "4")]
    pub request_end_block: ::core::option::Option<
        ::tendermint_proto::abci::RequestEndBlock,
    >,
    #[prost(message, optional, tag = "5")]
    pub response_end_block: ::core::option::Option<
        ::tendermint_proto::abci::ResponseEndBlock,
    >,
    #[prost(message, optional, tag = "6")]
    pub response_commit: ::core::option::Option<
        ::tendermint_proto::abci::ResponseCommit,
    >,
}
/// Nested message and enum types in `BlockMetadata`.
pub mod block_metadata {
    /// DeliverTx encapulate deliver tx request and response.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct DeliverTx {
        #[prost(message, optional, tag = "1")]
        pub request: ::core::option::Option<::tendermint_proto::abci::RequestDeliverTx>,
        #[prost(message, optional, tag = "2")]
        pub response: ::core::option::Option<
            ::tendermint_proto::abci::ResponseDeliverTx,
        >,
    }
}
