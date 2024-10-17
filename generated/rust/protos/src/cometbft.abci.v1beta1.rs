// @generated
/// Request represents a request to the ABCI application.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Request {
    /// Sum of all possible messages.
    #[prost(
        oneof = "request::Value",
        tags = "1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15"
    )]
    pub value: ::core::option::Option<request::Value>,
}
/// Nested message and enum types in `Request`.
pub mod request {
    /// Sum of all possible messages.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag = "1")]
        Echo(super::RequestEcho),
        #[prost(message, tag = "2")]
        Flush(super::RequestFlush),
        #[prost(message, tag = "3")]
        Info(super::RequestInfo),
        #[prost(message, tag = "4")]
        SetOption(super::RequestSetOption),
        #[prost(message, tag = "5")]
        InitChain(super::RequestInitChain),
        #[prost(message, tag = "6")]
        Query(super::RequestQuery),
        #[prost(message, tag = "7")]
        BeginBlock(super::RequestBeginBlock),
        #[prost(message, tag = "8")]
        CheckTx(super::RequestCheckTx),
        #[prost(message, tag = "9")]
        DeliverTx(super::RequestDeliverTx),
        #[prost(message, tag = "10")]
        EndBlock(super::RequestEndBlock),
        #[prost(message, tag = "11")]
        Commit(super::RequestCommit),
        #[prost(message, tag = "12")]
        ListSnapshots(super::RequestListSnapshots),
        #[prost(message, tag = "13")]
        OfferSnapshot(super::RequestOfferSnapshot),
        #[prost(message, tag = "14")]
        LoadSnapshotChunk(super::RequestLoadSnapshotChunk),
        #[prost(message, tag = "15")]
        ApplySnapshotChunk(super::RequestApplySnapshotChunk),
    }
}
impl ::prost::Name for Request {
    const NAME: &'static str = "Request";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// RequestEcho is a request to "echo" the given string.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestEcho {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}
impl ::prost::Name for RequestEcho {
    const NAME: &'static str = "RequestEcho";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// RequestFlush is a request to flush the write buffer.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestFlush {}
impl ::prost::Name for RequestFlush {
    const NAME: &'static str = "RequestFlush";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// RequestInfo is a request for the ABCI application version.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestInfo {
    #[prost(string, tag = "1")]
    pub version: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub block_version: u64,
    #[prost(uint64, tag = "3")]
    pub p2p_version: u64,
}
impl ::prost::Name for RequestInfo {
    const NAME: &'static str = "RequestInfo";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// nondeterministic
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestSetOption {
    #[prost(string, tag = "1")]
    pub key: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub value: ::prost::alloc::string::String,
}
impl ::prost::Name for RequestSetOption {
    const NAME: &'static str = "RequestSetOption";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// RequestInitChain is a request to initialize the blockchain.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestInitChain {
    #[prost(message, optional, tag = "1")]
    pub time: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub consensus_params: ::core::option::Option<ConsensusParams>,
    #[prost(message, repeated, tag = "4")]
    pub validators: ::prost::alloc::vec::Vec<ValidatorUpdate>,
    #[prost(bytes = "vec", tag = "5")]
    pub app_state_bytes: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag = "6")]
    pub initial_height: i64,
}
impl ::prost::Name for RequestInitChain {
    const NAME: &'static str = "RequestInitChain";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// RequestQuery is a request to query the application state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestQuery {
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "2")]
    pub path: ::prost::alloc::string::String,
    #[prost(int64, tag = "3")]
    pub height: i64,
    #[prost(bool, tag = "4")]
    pub prove: bool,
}
impl ::prost::Name for RequestQuery {
    const NAME: &'static str = "RequestQuery";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// RequestBeginBlock indicates the beginning of committing the block.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestBeginBlock {
    #[prost(bytes = "vec", tag = "1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub header: ::core::option::Option<super::super::types::v1beta1::Header>,
    #[prost(message, optional, tag = "3")]
    pub last_commit_info: ::core::option::Option<LastCommitInfo>,
    #[prost(message, repeated, tag = "4")]
    pub byzantine_validators: ::prost::alloc::vec::Vec<Evidence>,
}
impl ::prost::Name for RequestBeginBlock {
    const NAME: &'static str = "RequestBeginBlock";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// RequestCheckTx is a request to check the transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestCheckTx {
    #[prost(bytes = "vec", tag = "1")]
    pub tx: ::prost::alloc::vec::Vec<u8>,
    #[prost(enumeration = "CheckTxType", tag = "2")]
    pub r#type: i32,
}
impl ::prost::Name for RequestCheckTx {
    const NAME: &'static str = "RequestCheckTx";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// RequestDeliverTx is a request to apply the transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestDeliverTx {
    #[prost(bytes = "vec", tag = "1")]
    pub tx: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for RequestDeliverTx {
    const NAME: &'static str = "RequestDeliverTx";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// RequestEndBlock indicates the end of committing the block.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestEndBlock {
    #[prost(int64, tag = "1")]
    pub height: i64,
}
impl ::prost::Name for RequestEndBlock {
    const NAME: &'static str = "RequestEndBlock";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// RequestCommit is a request to commit the pending application state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestCommit {}
impl ::prost::Name for RequestCommit {
    const NAME: &'static str = "RequestCommit";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// lists available snapshots
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestListSnapshots {}
impl ::prost::Name for RequestListSnapshots {
    const NAME: &'static str = "RequestListSnapshots";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// offers a snapshot to the application
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestOfferSnapshot {
    /// snapshot offered by peers
    #[prost(message, optional, tag = "1")]
    pub snapshot: ::core::option::Option<Snapshot>,
    /// light client-verified app hash for snapshot height
    #[prost(bytes = "vec", tag = "2")]
    pub app_hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for RequestOfferSnapshot {
    const NAME: &'static str = "RequestOfferSnapshot";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// loads a snapshot chunk
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestLoadSnapshotChunk {
    #[prost(uint64, tag = "1")]
    pub height: u64,
    #[prost(uint32, tag = "2")]
    pub format: u32,
    #[prost(uint32, tag = "3")]
    pub chunk: u32,
}
impl ::prost::Name for RequestLoadSnapshotChunk {
    const NAME: &'static str = "RequestLoadSnapshotChunk";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// Applies a snapshot chunk
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestApplySnapshotChunk {
    #[prost(uint32, tag = "1")]
    pub index: u32,
    #[prost(bytes = "vec", tag = "2")]
    pub chunk: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "3")]
    pub sender: ::prost::alloc::string::String,
}
impl ::prost::Name for RequestApplySnapshotChunk {
    const NAME: &'static str = "RequestApplySnapshotChunk";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// Response represents a response from the ABCI application.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Response {
    /// Sum of all possible messages.
    #[prost(
        oneof = "response::Value",
        tags = "1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16"
    )]
    pub value: ::core::option::Option<response::Value>,
}
/// Nested message and enum types in `Response`.
pub mod response {
    /// Sum of all possible messages.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag = "1")]
        Exception(super::ResponseException),
        #[prost(message, tag = "2")]
        Echo(super::ResponseEcho),
        #[prost(message, tag = "3")]
        Flush(super::ResponseFlush),
        #[prost(message, tag = "4")]
        Info(super::ResponseInfo),
        #[prost(message, tag = "5")]
        SetOption(super::ResponseSetOption),
        #[prost(message, tag = "6")]
        InitChain(super::ResponseInitChain),
        #[prost(message, tag = "7")]
        Query(super::ResponseQuery),
        #[prost(message, tag = "8")]
        BeginBlock(super::ResponseBeginBlock),
        #[prost(message, tag = "9")]
        CheckTx(super::ResponseCheckTx),
        #[prost(message, tag = "10")]
        DeliverTx(super::ResponseDeliverTx),
        #[prost(message, tag = "11")]
        EndBlock(super::ResponseEndBlock),
        #[prost(message, tag = "12")]
        Commit(super::ResponseCommit),
        #[prost(message, tag = "13")]
        ListSnapshots(super::ResponseListSnapshots),
        #[prost(message, tag = "14")]
        OfferSnapshot(super::ResponseOfferSnapshot),
        #[prost(message, tag = "15")]
        LoadSnapshotChunk(super::ResponseLoadSnapshotChunk),
        #[prost(message, tag = "16")]
        ApplySnapshotChunk(super::ResponseApplySnapshotChunk),
    }
}
impl ::prost::Name for Response {
    const NAME: &'static str = "Response";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// nondeterministic
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseException {
    #[prost(string, tag = "1")]
    pub error: ::prost::alloc::string::String,
}
impl ::prost::Name for ResponseException {
    const NAME: &'static str = "ResponseException";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// ResponseEcho indicates that the connection is still alive.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseEcho {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}
impl ::prost::Name for ResponseEcho {
    const NAME: &'static str = "ResponseEcho";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// ResponseFlush indicates that the ABCI application state was flushed?
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseFlush {}
impl ::prost::Name for ResponseFlush {
    const NAME: &'static str = "ResponseFlush";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// ResponseInfo contains the ABCI application version information.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseInfo {
    #[prost(string, tag = "1")]
    pub data: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub version: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub app_version: u64,
    #[prost(int64, tag = "4")]
    pub last_block_height: i64,
    #[prost(bytes = "vec", tag = "5")]
    pub last_block_app_hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ResponseInfo {
    const NAME: &'static str = "ResponseInfo";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// nondeterministic
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseSetOption {
    #[prost(uint32, tag = "1")]
    pub code: u32,
    /// bytes data = 2;
    #[prost(string, tag = "3")]
    pub log: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub info: ::prost::alloc::string::String,
}
impl ::prost::Name for ResponseSetOption {
    const NAME: &'static str = "ResponseSetOption";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// ResponseInitChain contains the ABCI application's hash and updates to the
/// validator set and/or the consensus params, if any.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseInitChain {
    #[prost(message, optional, tag = "1")]
    pub consensus_params: ::core::option::Option<ConsensusParams>,
    #[prost(message, repeated, tag = "2")]
    pub validators: ::prost::alloc::vec::Vec<ValidatorUpdate>,
    #[prost(bytes = "vec", tag = "3")]
    pub app_hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ResponseInitChain {
    const NAME: &'static str = "ResponseInitChain";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// ResponseQuery contains the ABCI application data along with a proof.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseQuery {
    #[prost(uint32, tag = "1")]
    pub code: u32,
    /// bytes data = 2; // use "value" instead.
    ///
    /// nondeterministic
    #[prost(string, tag = "3")]
    pub log: ::prost::alloc::string::String,
    /// nondeterministic
    #[prost(string, tag = "4")]
    pub info: ::prost::alloc::string::String,
    #[prost(int64, tag = "5")]
    pub index: i64,
    #[prost(bytes = "vec", tag = "6")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "7")]
    pub value: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "8")]
    pub proof_ops: ::core::option::Option<super::super::crypto::v1::ProofOps>,
    #[prost(int64, tag = "9")]
    pub height: i64,
    #[prost(string, tag = "10")]
    pub codespace: ::prost::alloc::string::String,
}
impl ::prost::Name for ResponseQuery {
    const NAME: &'static str = "ResponseQuery";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// ResponseBeginBlock contains a list of block-level events.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseBeginBlock {
    #[prost(message, repeated, tag = "1")]
    pub events: ::prost::alloc::vec::Vec<Event>,
}
impl ::prost::Name for ResponseBeginBlock {
    const NAME: &'static str = "ResponseBeginBlock";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// ResponseCheckTx shows if the transaction was deemed valid by the ABCI
/// application.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseCheckTx {
    #[prost(uint32, tag = "1")]
    pub code: u32,
    #[prost(bytes = "vec", tag = "2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    /// nondeterministic
    #[prost(string, tag = "3")]
    pub log: ::prost::alloc::string::String,
    /// nondeterministic
    #[prost(string, tag = "4")]
    pub info: ::prost::alloc::string::String,
    #[prost(int64, tag = "5")]
    pub gas_wanted: i64,
    #[prost(int64, tag = "6")]
    pub gas_used: i64,
    #[prost(message, repeated, tag = "7")]
    pub events: ::prost::alloc::vec::Vec<Event>,
    #[prost(string, tag = "8")]
    pub codespace: ::prost::alloc::string::String,
    #[prost(string, tag = "9")]
    pub sender: ::prost::alloc::string::String,
    #[prost(int64, tag = "10")]
    pub priority: i64,
    /// mempool_error is set by CometBFT.
    /// ABCI applications creating a ResponseCheckTX should not set mempool_error.
    #[prost(string, tag = "11")]
    pub mempool_error: ::prost::alloc::string::String,
}
impl ::prost::Name for ResponseCheckTx {
    const NAME: &'static str = "ResponseCheckTx";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// ResponseDeliverTx contains a result of committing the given transaction and a
/// list of events.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseDeliverTx {
    #[prost(uint32, tag = "1")]
    pub code: u32,
    #[prost(bytes = "vec", tag = "2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    /// nondeterministic
    #[prost(string, tag = "3")]
    pub log: ::prost::alloc::string::String,
    /// nondeterministic
    #[prost(string, tag = "4")]
    pub info: ::prost::alloc::string::String,
    #[prost(int64, tag = "5")]
    pub gas_wanted: i64,
    #[prost(int64, tag = "6")]
    pub gas_used: i64,
    /// nondeterministic
    #[prost(message, repeated, tag = "7")]
    pub events: ::prost::alloc::vec::Vec<Event>,
    #[prost(string, tag = "8")]
    pub codespace: ::prost::alloc::string::String,
}
impl ::prost::Name for ResponseDeliverTx {
    const NAME: &'static str = "ResponseDeliverTx";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// ResponseEndBlock contains updates to consensus params and/or validator set changes, if any.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseEndBlock {
    #[prost(message, repeated, tag = "1")]
    pub validator_updates: ::prost::alloc::vec::Vec<ValidatorUpdate>,
    #[prost(message, optional, tag = "2")]
    pub consensus_param_updates: ::core::option::Option<ConsensusParams>,
    #[prost(message, repeated, tag = "3")]
    pub events: ::prost::alloc::vec::Vec<Event>,
}
impl ::prost::Name for ResponseEndBlock {
    const NAME: &'static str = "ResponseEndBlock";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// ResponseCommit indicates how much blocks should CometBFT retain.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseCommit {
    /// reserve 1
    #[prost(bytes = "vec", tag = "2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag = "3")]
    pub retain_height: i64,
}
impl ::prost::Name for ResponseCommit {
    const NAME: &'static str = "ResponseCommit";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// ResponseListSnapshots contains the list of snapshots.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseListSnapshots {
    #[prost(message, repeated, tag = "1")]
    pub snapshots: ::prost::alloc::vec::Vec<Snapshot>,
}
impl ::prost::Name for ResponseListSnapshots {
    const NAME: &'static str = "ResponseListSnapshots";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// ResponseOfferSnapshot indicates the ABCI application decision whenever to
/// provide a snapshot to the requester or not.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseOfferSnapshot {
    #[prost(enumeration = "response_offer_snapshot::Result", tag = "1")]
    pub result: i32,
}
/// Nested message and enum types in `ResponseOfferSnapshot`.
pub mod response_offer_snapshot {
    /// The status code.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Result {
        /// Unknown result, abort all snapshot restoration
        Unknown = 0,
        /// Snapshot accepted, apply chunks
        Accept = 1,
        /// Abort all snapshot restoration
        Abort = 2,
        /// Reject this specific snapshot, try others
        Reject = 3,
        /// Reject all snapshots of this format, try others
        RejectFormat = 4,
        /// Reject all snapshots from the sender(s), try others
        RejectSender = 5,
    }
    impl Result {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Result::Unknown => "UNKNOWN",
                Result::Accept => "ACCEPT",
                Result::Abort => "ABORT",
                Result::Reject => "REJECT",
                Result::RejectFormat => "REJECT_FORMAT",
                Result::RejectSender => "REJECT_SENDER",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "UNKNOWN" => Some(Self::Unknown),
                "ACCEPT" => Some(Self::Accept),
                "ABORT" => Some(Self::Abort),
                "REJECT" => Some(Self::Reject),
                "REJECT_FORMAT" => Some(Self::RejectFormat),
                "REJECT_SENDER" => Some(Self::RejectSender),
                _ => None,
            }
        }
    }
}
impl ::prost::Name for ResponseOfferSnapshot {
    const NAME: &'static str = "ResponseOfferSnapshot";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// ResponseLoadSnapshotChunk returns a snapshot's chunk.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseLoadSnapshotChunk {
    #[prost(bytes = "vec", tag = "1")]
    pub chunk: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ResponseLoadSnapshotChunk {
    const NAME: &'static str = "ResponseLoadSnapshotChunk";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// ResponseApplySnapshotChunk returns a result of applying the specified chunk.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseApplySnapshotChunk {
    #[prost(enumeration = "response_apply_snapshot_chunk::Result", tag = "1")]
    pub result: i32,
    /// Chunks to refetch and reapply
    #[prost(uint32, repeated, tag = "2")]
    pub refetch_chunks: ::prost::alloc::vec::Vec<u32>,
    /// Chunk senders to reject and ban
    #[prost(string, repeated, tag = "3")]
    pub reject_senders: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// Nested message and enum types in `ResponseApplySnapshotChunk`.
pub mod response_apply_snapshot_chunk {
    /// The status code.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Result {
        /// Unknown result, abort all snapshot restoration
        Unknown = 0,
        /// Chunk successfully accepted
        Accept = 1,
        /// Abort all snapshot restoration
        Abort = 2,
        /// Retry chunk (combine with refetch and reject)
        Retry = 3,
        /// Retry snapshot (combine with refetch and reject)
        RetrySnapshot = 4,
        /// Reject this snapshot, try others
        RejectSnapshot = 5,
    }
    impl Result {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Result::Unknown => "UNKNOWN",
                Result::Accept => "ACCEPT",
                Result::Abort => "ABORT",
                Result::Retry => "RETRY",
                Result::RetrySnapshot => "RETRY_SNAPSHOT",
                Result::RejectSnapshot => "REJECT_SNAPSHOT",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "UNKNOWN" => Some(Self::Unknown),
                "ACCEPT" => Some(Self::Accept),
                "ABORT" => Some(Self::Abort),
                "RETRY" => Some(Self::Retry),
                "RETRY_SNAPSHOT" => Some(Self::RetrySnapshot),
                "REJECT_SNAPSHOT" => Some(Self::RejectSnapshot),
                _ => None,
            }
        }
    }
}
impl ::prost::Name for ResponseApplySnapshotChunk {
    const NAME: &'static str = "ResponseApplySnapshotChunk";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// ConsensusParams contains all consensus-relevant parameters
/// that can be adjusted by the abci app
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusParams {
    #[prost(message, optional, tag = "1")]
    pub block: ::core::option::Option<BlockParams>,
    #[prost(message, optional, tag = "2")]
    pub evidence: ::core::option::Option<super::super::types::v1beta1::EvidenceParams>,
    #[prost(message, optional, tag = "3")]
    pub validator: ::core::option::Option<super::super::types::v1beta1::ValidatorParams>,
    #[prost(message, optional, tag = "4")]
    pub version: ::core::option::Option<super::super::types::v1beta1::VersionParams>,
}
impl ::prost::Name for ConsensusParams {
    const NAME: &'static str = "ConsensusParams";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// BlockParams contains limits on the block size.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockParams {
    /// Note: must be greater than 0
    #[prost(int64, tag = "1")]
    pub max_bytes: i64,
    /// Note: must be greater or equal to -1
    #[prost(int64, tag = "2")]
    pub max_gas: i64,
}
impl ::prost::Name for BlockParams {
    const NAME: &'static str = "BlockParams";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// LastCommitInfo contains votes for the particular round.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LastCommitInfo {
    #[prost(int32, tag = "1")]
    pub round: i32,
    #[prost(message, repeated, tag = "2")]
    pub votes: ::prost::alloc::vec::Vec<VoteInfo>,
}
impl ::prost::Name for LastCommitInfo {
    const NAME: &'static str = "LastCommitInfo";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// Event allows application developers to attach additional information to
/// ResponseBeginBlock, ResponseEndBlock, ResponseCheckTx and ResponseDeliverTx.
/// Later, transactions may be queried using these events.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Event {
    #[prost(string, tag = "1")]
    pub r#type: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub attributes: ::prost::alloc::vec::Vec<EventAttribute>,
}
impl ::prost::Name for Event {
    const NAME: &'static str = "Event";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// EventAttribute is a single key-value pair, associated with an event.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventAttribute {
    #[prost(bytes = "vec", tag = "1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub value: ::prost::alloc::vec::Vec<u8>,
    /// nondeterministic
    #[prost(bool, tag = "3")]
    pub index: bool,
}
impl ::prost::Name for EventAttribute {
    const NAME: &'static str = "EventAttribute";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// TxResult contains results of executing the transaction.
///
/// One usage is indexing transaction results.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxResult {
    #[prost(int64, tag = "1")]
    pub height: i64,
    #[prost(uint32, tag = "2")]
    pub index: u32,
    #[prost(bytes = "vec", tag = "3")]
    pub tx: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "4")]
    pub result: ::core::option::Option<ResponseDeliverTx>,
}
impl ::prost::Name for TxResult {
    const NAME: &'static str = "TxResult";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// Validator in the validator set.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Validator {
    /// The first 20 bytes of SHA256(public key)
    #[prost(bytes = "vec", tag = "1")]
    pub address: ::prost::alloc::vec::Vec<u8>,
    /// PubKey pub_key = 2 \[(gogoproto.nullable)=false\];
    ///
    /// The voting power
    #[prost(int64, tag = "3")]
    pub power: i64,
}
impl ::prost::Name for Validator {
    const NAME: &'static str = "Validator";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// ValidatorUpdate is a singular update to a validator set.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorUpdate {
    #[prost(message, optional, tag = "1")]
    pub pub_key: ::core::option::Option<super::super::crypto::v1::PublicKey>,
    #[prost(int64, tag = "2")]
    pub power: i64,
}
impl ::prost::Name for ValidatorUpdate {
    const NAME: &'static str = "ValidatorUpdate";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// VoteInfo contains the information about the vote.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VoteInfo {
    #[prost(message, optional, tag = "1")]
    pub validator: ::core::option::Option<Validator>,
    #[prost(bool, tag = "2")]
    pub signed_last_block: bool,
}
impl ::prost::Name for VoteInfo {
    const NAME: &'static str = "VoteInfo";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// Evidence of a misbehavior committed by a validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Evidence {
    #[prost(enumeration = "EvidenceType", tag = "1")]
    pub r#type: i32,
    /// The offending validator
    #[prost(message, optional, tag = "2")]
    pub validator: ::core::option::Option<Validator>,
    /// The height when the offense occurred
    #[prost(int64, tag = "3")]
    pub height: i64,
    /// The corresponding time where the offense occurred
    #[prost(message, optional, tag = "4")]
    pub time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// Total voting power of the validator set in case the ABCI application does
    /// not store historical validators.
    /// <https://github.com/tendermint/tendermint/issues/4581>
    #[prost(int64, tag = "5")]
    pub total_voting_power: i64,
}
impl ::prost::Name for Evidence {
    const NAME: &'static str = "Evidence";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// Snapshot of the ABCI application state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Snapshot {
    /// The height at which the snapshot was taken
    #[prost(uint64, tag = "1")]
    pub height: u64,
    /// The application-specific snapshot format
    #[prost(uint32, tag = "2")]
    pub format: u32,
    /// Number of chunks in the snapshot
    #[prost(uint32, tag = "3")]
    pub chunks: u32,
    /// Arbitrary snapshot hash, equal only if identical
    #[prost(bytes = "vec", tag = "4")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    /// Arbitrary application metadata
    #[prost(bytes = "vec", tag = "5")]
    pub metadata: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for Snapshot {
    const NAME: &'static str = "Snapshot";
    const PACKAGE: &'static str = "cometbft.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta1.{}", Self::NAME)
    }
}
/// Type of the transaction check request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum CheckTxType {
    /// New
    New = 0,
    /// Recheck (2nd, 3rd, etc.)
    Recheck = 1,
}
impl CheckTxType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            CheckTxType::New => "NEW",
            CheckTxType::Recheck => "RECHECK",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "NEW" => Some(Self::New),
            "RECHECK" => Some(Self::Recheck),
            _ => None,
        }
    }
}
/// The type of evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum EvidenceType {
    /// Unknown
    Unknown = 0,
    /// Duplicate vote
    DuplicateVote = 1,
    /// Light client attack
    LightClientAttack = 2,
}
impl EvidenceType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            EvidenceType::Unknown => "UNKNOWN",
            EvidenceType::DuplicateVote => "DUPLICATE_VOTE",
            EvidenceType::LightClientAttack => "LIGHT_CLIENT_ATTACK",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "UNKNOWN" => Some(Self::Unknown),
            "DUPLICATE_VOTE" => Some(Self::DuplicateVote),
            "LIGHT_CLIENT_ATTACK" => Some(Self::LightClientAttack),
            _ => None,
        }
    }
}
include!("cometbft.abci.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
