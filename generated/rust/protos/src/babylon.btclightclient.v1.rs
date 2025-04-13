// @generated
/// BTCHeaderInfo is a structure that contains all relevant information about a
/// BTC header
///   - Full header bytes
///   - Header hash for easy retrieval
///   - Height of the header in the BTC chain
///   - Total work spent on the header. This is the sum of the work corresponding
///   to the header Bits field
///     and the total work of the header.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BtcHeaderInfo {
    #[prost(bytes = "vec", tag = "1")]
    pub header: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint32, tag = "3")]
    pub height: u32,
    #[prost(bytes = "vec", tag = "4")]
    pub work: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for BtcHeaderInfo {
    const NAME: &'static str = "BTCHeaderInfo";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// The header included in the event is the block in the history
/// of the current mainchain to which we are rolling back to.
/// In other words, there is one rollback event emitted per re-org, to the
/// greatest common ancestor of the old and the new fork.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventBtcRollBack {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<BtcHeaderInfo>,
    #[prost(message, optional, tag = "2")]
    pub rollback_from: ::core::option::Option<BtcHeaderInfo>,
}
impl ::prost::Name for EventBtcRollBack {
    const NAME: &'static str = "EventBTCRollBack";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// EventBTCRollForward is emitted on Msg/InsertHeader
/// The header included in the event is the one the main chain is extended with.
/// In the event of a reorg, each block on the new fork that comes after
/// the greatest common ancestor will have a corresponding roll forward event.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventBtcRollForward {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<BtcHeaderInfo>,
}
impl ::prost::Name for EventBtcRollForward {
    const NAME: &'static str = "EventBTCRollForward";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// EventBTCHeaderInserted is emitted on Msg/InsertHeader
/// The header included in the event is the one that was added to the
/// on chain BTC storage.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventBtcHeaderInserted {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<BtcHeaderInfo>,
}
impl ::prost::Name for EventBtcHeaderInserted {
    const NAME: &'static str = "EventBTCHeaderInserted";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// Params defines the parameters for the module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// List of addresses which are allowed to insert headers to btc light client
    /// if the list is empty, any address can insert headers
    #[prost(string, repeated, tag = "1")]
    pub insert_headers_allow_list: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// GenesisState defines the btclightclient module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
    #[prost(message, repeated, tag = "2")]
    pub btc_headers: ::prost::alloc::vec::Vec<BtcHeaderInfo>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// QueryParamsRequest is the request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// QueryParamsResponse is the response type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// params holds all the parameters of this module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for QueryParamsResponse {
    const NAME: &'static str = "QueryParamsResponse";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// QueryHashesRequest is request type for the Query/Hashes RPC method.
/// It involves retrieving all hashes that are maintained by the module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryHashesRequest {
    #[prost(message, optional, tag = "1")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryHashesRequest {
    const NAME: &'static str = "QueryHashesRequest";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// QueryHashesResponse is response type for the Query/Hashes RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryHashesResponse {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub hashes: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryHashesResponse {
    const NAME: &'static str = "QueryHashesResponse";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// QueryContainsRequest is request type for the Query/Contains RPC method.
/// It involves checking whether a hash is maintained by the module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryContainsRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for QueryContainsRequest {
    const NAME: &'static str = "QueryContainsRequest";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// QueryContainsResponse is response type for the Query/Contains RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryContainsResponse {
    #[prost(bool, tag = "1")]
    pub contains: bool,
}
impl ::prost::Name for QueryContainsResponse {
    const NAME: &'static str = "QueryContainsResponse";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// QueryContainsRequest is request type for the temporary Query/ContainsBytes
/// RPC method. It involves checking whether a hash is maintained by the module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryContainsBytesRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for QueryContainsBytesRequest {
    const NAME: &'static str = "QueryContainsBytesRequest";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// QueryContainsResponse is response type for the temporary Query/ContainsBytes
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryContainsBytesResponse {
    #[prost(bool, tag = "1")]
    pub contains: bool,
}
impl ::prost::Name for QueryContainsBytesResponse {
    const NAME: &'static str = "QueryContainsBytesResponse";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// QueryMainChainRequest is request type for the Query/MainChain RPC method.
/// It involves retrieving the canonical chain maintained by the module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryMainChainRequest {
    #[prost(message, optional, tag = "1")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryMainChainRequest {
    const NAME: &'static str = "QueryMainChainRequest";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// QueryMainChainResponse is response type for the Query/MainChain RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryMainChainResponse {
    #[prost(message, repeated, tag = "1")]
    pub headers: ::prost::alloc::vec::Vec<BtcHeaderInfoResponse>,
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryMainChainResponse {
    const NAME: &'static str = "QueryMainChainResponse";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// QueryTipRequest is the request type for the Query/Tip RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTipRequest {}
impl ::prost::Name for QueryTipRequest {
    const NAME: &'static str = "QueryTipRequest";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// QueryTipResponse is the response type for the Query/Tip RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTipResponse {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<BtcHeaderInfoResponse>,
}
impl ::prost::Name for QueryTipResponse {
    const NAME: &'static str = "QueryTipResponse";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// QueryBaseHeaderRequest is the request type for the Query/BaseHeader RPC
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBaseHeaderRequest {}
impl ::prost::Name for QueryBaseHeaderRequest {
    const NAME: &'static str = "QueryBaseHeaderRequest";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// QueryBaseHeaderResponse is the response type for the Query/BaseHeader RPC
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBaseHeaderResponse {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<BtcHeaderInfoResponse>,
}
impl ::prost::Name for QueryBaseHeaderResponse {
    const NAME: &'static str = "QueryBaseHeaderResponse";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// QueryMainChainDepthRequest is the request type for the Query/MainChainDepth RPC
/// it contains hex encoded hash of btc block header as parameter
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryHeaderDepthRequest {
    #[prost(string, tag = "1")]
    pub hash: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryHeaderDepthRequest {
    const NAME: &'static str = "QueryHeaderDepthRequest";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// QueryMainChainDepthResponse is the response type for the Query/MainChainDepth RPC
/// it contains depth of the block in main chain
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryHeaderDepthResponse {
    #[prost(uint32, tag = "1")]
    pub depth: u32,
}
impl ::prost::Name for QueryHeaderDepthResponse {
    const NAME: &'static str = "QueryHeaderDepthResponse";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// BTCHeaderInfoResponse is a structure that contains all relevant information about a
/// BTC header response
///   - Full header as string hex.
///   - Header hash for easy retrieval as string hex.
///   - Height of the header in the BTC chain.
///   - Total work spent on the header. This is the sum of the work corresponding
///   to the header Bits field
///     and the total work of the header.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BtcHeaderInfoResponse {
    #[prost(string, tag = "1")]
    pub header_hex: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub hash_hex: ::prost::alloc::string::String,
    #[prost(uint32, tag = "3")]
    pub height: u32,
    /// Work is the sdkmath.Uint as string.
    #[prost(string, tag = "4")]
    pub work: ::prost::alloc::string::String,
}
impl ::prost::Name for BtcHeaderInfoResponse {
    const NAME: &'static str = "BTCHeaderInfoResponse";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// MsgInsertHeaders defines the message for multiple incoming header bytes
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgInsertHeaders {
    #[prost(string, tag = "1")]
    pub signer: ::prost::alloc::string::String,
    #[prost(bytes = "vec", repeated, tag = "2")]
    pub headers: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
impl ::prost::Name for MsgInsertHeaders {
    const NAME: &'static str = "MsgInsertHeaders";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// MsgInsertHeadersResponse defines the response for the InsertHeaders transaction
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgInsertHeadersResponse {}
impl ::prost::Name for MsgInsertHeadersResponse {
    const NAME: &'static str = "MsgInsertHeadersResponse";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// MsgUpdateParams defines a message for updating btc light client module parameters.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParams {
    /// authority is the address of the governance account.
    /// just FYI: cosmos.AddressString marks that this field should use type alias
    /// for AddressString instead of string, but the functionality is not yet implemented
    /// in cosmos-proto
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// params defines the btc light client parameters to update.
    ///
    /// NOTE: All parameters must be supplied.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for MsgUpdateParams {
    const NAME: &'static str = "MsgUpdateParams";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
/// MsgUpdateParamsResponse is the response to the MsgUpdateParams message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParamsResponse {}
impl ::prost::Name for MsgUpdateParamsResponse {
    const NAME: &'static str = "MsgUpdateParamsResponse";
    const PACKAGE: &'static str = "babylon.btclightclient.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btclightclient.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
