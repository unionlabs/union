// @generated
/// TxResponse defines a structure containing relevant tx data and metadata. The
/// tags are stringified and the log is JSON decoded.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxResponse {
    /// The block height
    #[prost(int64, tag = "1")]
    pub height: i64,
    /// The transaction hash.
    #[prost(string, tag = "2")]
    pub txhash: ::prost::alloc::string::String,
    /// Namespace for the Code
    #[prost(string, tag = "3")]
    pub codespace: ::prost::alloc::string::String,
    /// Response code.
    #[prost(uint32, tag = "4")]
    pub code: u32,
    /// Result bytes, if any.
    #[prost(string, tag = "5")]
    pub data: ::prost::alloc::string::String,
    /// The output of the application's logger (raw string). May be
    /// non-deterministic.
    #[prost(string, tag = "6")]
    pub raw_log: ::prost::alloc::string::String,
    /// The output of the application's logger (typed). May be non-deterministic.
    #[prost(message, repeated, tag = "7")]
    pub logs: ::prost::alloc::vec::Vec<AbciMessageLog>,
    /// Additional information. May be non-deterministic.
    #[prost(string, tag = "8")]
    pub info: ::prost::alloc::string::String,
    /// Amount of gas requested for transaction.
    #[prost(int64, tag = "9")]
    pub gas_wanted: i64,
    /// Amount of gas consumed by transaction.
    #[prost(int64, tag = "10")]
    pub gas_used: i64,
    /// The request transaction bytes.
    #[prost(message, optional, tag = "11")]
    pub tx: ::core::option::Option<::pbjson_types::Any>,
    /// Time of the previous block. For heights > 1, it's the weighted median of
    /// the timestamps of the valid votes in the block.LastCommit. For height == 1,
    /// it's genesis time.
    #[prost(string, tag = "12")]
    pub timestamp: ::prost::alloc::string::String,
    /// Events defines all the events emitted by processing a transaction. Note,
    /// these events include those emitted by processing all the messages and those
    /// emitted from the ante. Whereas Logs contains the events, with
    /// additional metadata, emitted only by processing the messages.
    ///
    /// Since: cosmos-sdk 0.42.11, 0.44.5, 0.45
    #[prost(message, repeated, tag = "13")]
    pub events: ::prost::alloc::vec::Vec<super::super::super::super::tendermint::abci::Event>,
}
impl ::prost::Name for TxResponse {
    const NAME: &'static str = "TxResponse";
    const PACKAGE: &'static str = "cosmos.base.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.abci.v1beta1.{}", Self::NAME)
    }
}
/// ABCIMessageLog defines a structure containing an indexed tx ABCI message log.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AbciMessageLog {
    #[prost(uint32, tag = "1")]
    pub msg_index: u32,
    #[prost(string, tag = "2")]
    pub log: ::prost::alloc::string::String,
    /// Events contains a slice of Event objects that were emitted during some
    /// execution.
    #[prost(message, repeated, tag = "3")]
    pub events: ::prost::alloc::vec::Vec<StringEvent>,
}
impl ::prost::Name for AbciMessageLog {
    const NAME: &'static str = "ABCIMessageLog";
    const PACKAGE: &'static str = "cosmos.base.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.abci.v1beta1.{}", Self::NAME)
    }
}
/// StringEvent defines en Event object wrapper where all the attributes
/// contain key/value pairs that are strings instead of raw bytes.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StringEvent {
    #[prost(string, tag = "1")]
    pub r#type: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub attributes: ::prost::alloc::vec::Vec<Attribute>,
}
impl ::prost::Name for StringEvent {
    const NAME: &'static str = "StringEvent";
    const PACKAGE: &'static str = "cosmos.base.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.abci.v1beta1.{}", Self::NAME)
    }
}
/// Attribute defines an attribute wrapper where the key and value are
/// strings instead of raw bytes.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Attribute {
    #[prost(string, tag = "1")]
    pub key: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub value: ::prost::alloc::string::String,
}
impl ::prost::Name for Attribute {
    const NAME: &'static str = "Attribute";
    const PACKAGE: &'static str = "cosmos.base.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.abci.v1beta1.{}", Self::NAME)
    }
}
/// GasInfo defines tx execution gas context.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GasInfo {
    /// GasWanted is the maximum units of work we allow this tx to perform.
    #[prost(uint64, tag = "1")]
    pub gas_wanted: u64,
    /// GasUsed is the amount of gas actually consumed.
    #[prost(uint64, tag = "2")]
    pub gas_used: u64,
}
impl ::prost::Name for GasInfo {
    const NAME: &'static str = "GasInfo";
    const PACKAGE: &'static str = "cosmos.base.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.abci.v1beta1.{}", Self::NAME)
    }
}
/// Result is the union of ResponseFormat and ResponseCheckTx.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Result {
    /// Data is any data returned from message or handler execution. It MUST be
    /// length prefixed in order to separate data from multiple message executions.
    /// Deprecated. This field is still populated, but prefer msg_response instead
    /// because it also contains the Msg response typeURL.
    #[deprecated]
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    /// Log contains the log information from message or handler execution.
    #[prost(string, tag = "2")]
    pub log: ::prost::alloc::string::String,
    // /// Events contains a slice of Event objects that were emitted during message
    // /// or handler execution.
    // #[prost(message, repeated, tag = "3")]
    // pub events: ::prost::alloc::vec::Vec<super::super::super::super::tendermint::abci::Event>,
    /// msg_responses contains the Msg handler responses type packed in Anys.
    ///
    /// Since: cosmos-sdk 0.46
    #[prost(message, repeated, tag = "4")]
    pub msg_responses: ::prost::alloc::vec::Vec<::pbjson_types::Any>,
}
impl ::prost::Name for Result {
    const NAME: &'static str = "Result";
    const PACKAGE: &'static str = "cosmos.base.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.abci.v1beta1.{}", Self::NAME)
    }
}
/// SimulationResponse defines the response generated when a transaction is
/// successfully simulated.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SimulationResponse {
    #[prost(message, optional, tag = "1")]
    pub gas_info: ::core::option::Option<GasInfo>,
    #[prost(message, optional, tag = "2")]
    pub result: ::core::option::Option<Result>,
}
impl ::prost::Name for SimulationResponse {
    const NAME: &'static str = "SimulationResponse";
    const PACKAGE: &'static str = "cosmos.base.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.abci.v1beta1.{}", Self::NAME)
    }
}
/// MsgData defines the data returned in a Result object during message
/// execution.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgData {
    #[prost(string, tag = "1")]
    pub msg_type: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgData {
    const NAME: &'static str = "MsgData";
    const PACKAGE: &'static str = "cosmos.base.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.abci.v1beta1.{}", Self::NAME)
    }
}
/// TxMsgData defines a list of MsgData. A transaction will have a MsgData object
/// for each message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxMsgData {
    /// data field is deprecated and not populated.
    #[deprecated]
    #[prost(message, repeated, tag = "1")]
    pub data: ::prost::alloc::vec::Vec<MsgData>,
    /// msg_responses contains the Msg handler responses packed into Anys.
    ///
    /// Since: cosmos-sdk 0.46
    #[prost(message, repeated, tag = "2")]
    pub msg_responses: ::prost::alloc::vec::Vec<::pbjson_types::Any>,
}
impl ::prost::Name for TxMsgData {
    const NAME: &'static str = "TxMsgData";
    const PACKAGE: &'static str = "cosmos.base.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.abci.v1beta1.{}", Self::NAME)
    }
}
/// SearchTxsResult defines a structure for querying txs pageable
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SearchTxsResult {
    /// Count of all txs
    #[prost(uint64, tag = "1")]
    pub total_count: u64,
    /// Count of txs in current page
    #[prost(uint64, tag = "2")]
    pub count: u64,
    /// Index of current page, start from 1
    #[prost(uint64, tag = "3")]
    pub page_number: u64,
    /// Count of total pages
    #[prost(uint64, tag = "4")]
    pub page_total: u64,
    /// Max count txs per page
    #[prost(uint64, tag = "5")]
    pub limit: u64,
    /// List of txs in current page
    #[prost(message, repeated, tag = "6")]
    pub txs: ::prost::alloc::vec::Vec<TxResponse>,
}
impl ::prost::Name for SearchTxsResult {
    const NAME: &'static str = "SearchTxsResult";
    const PACKAGE: &'static str = "cosmos.base.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.abci.v1beta1.{}", Self::NAME)
    }
}
/// SearchBlocksResult defines a structure for querying blocks pageable
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SearchBlocksResult {
    /// Count of all blocks
    #[prost(int64, tag = "1")]
    pub total_count: i64,
    /// Count of blocks in current page
    #[prost(int64, tag = "2")]
    pub count: i64,
    /// Index of current page, start from 1
    #[prost(int64, tag = "3")]
    pub page_number: i64,
    /// Count of total pages
    #[prost(int64, tag = "4")]
    pub page_total: i64,
    /// Max count blocks per page
    #[prost(int64, tag = "5")]
    pub limit: i64,
    /// List of blocks in current page
    #[prost(message, repeated, tag = "6")]
    pub blocks: ::prost::alloc::vec::Vec<super::super::super::super::tendermint::types::Block>,
}
impl ::prost::Name for SearchBlocksResult {
    const NAME: &'static str = "SearchBlocksResult";
    const PACKAGE: &'static str = "cosmos.base.abci.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.abci.v1beta1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
