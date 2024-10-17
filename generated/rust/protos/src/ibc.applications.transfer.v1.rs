// @generated
/// Allocation defines the spend limit for a particular port and channel
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Allocation {
    /// the port on which the packet will be sent
    #[prost(string, tag = "1")]
    pub source_port: ::prost::alloc::string::String,
    /// the channel by which the packet will be sent
    #[prost(string, tag = "2")]
    pub source_channel: ::prost::alloc::string::String,
    /// spend limitation on the channel
    #[prost(message, repeated, tag = "3")]
    pub spend_limit:
        ::prost::alloc::vec::Vec<super::super::super::super::cosmos::base::v1beta1::Coin>,
    /// allow list of receivers, an empty allow list permits any receiver address
    #[prost(string, repeated, tag = "4")]
    pub allow_list: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// allow list of memo strings, an empty list prohibits all memo strings;
    /// a list only with "*" permits any memo string
    #[prost(string, repeated, tag = "5")]
    pub allowed_packet_data: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for Allocation {
    const NAME: &'static str = "Allocation";
    const PACKAGE: &'static str = "ibc.applications.transfer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.transfer.v1.{}", Self::NAME)
    }
}
/// TransferAuthorization allows the grantee to spend up to spend_limit coins from
/// the granter's account for ibc transfer on a specific channel
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransferAuthorization {
    /// port and channel amounts
    #[prost(message, repeated, tag = "1")]
    pub allocations: ::prost::alloc::vec::Vec<Allocation>,
}
impl ::prost::Name for TransferAuthorization {
    const NAME: &'static str = "TransferAuthorization";
    const PACKAGE: &'static str = "ibc.applications.transfer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.transfer.v1.{}", Self::NAME)
    }
}
/// DenomTrace contains the base denomination for ICS20 fungible tokens and the
/// source tracing information path.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DenomTrace {
    /// path defines the chain of port/channel identifiers used for tracing the
    /// source of the fungible token.
    #[prost(string, tag = "1")]
    pub path: ::prost::alloc::string::String,
    /// base denomination of the relayed fungible token.
    #[prost(string, tag = "2")]
    pub base_denom: ::prost::alloc::string::String,
}
impl ::prost::Name for DenomTrace {
    const NAME: &'static str = "DenomTrace";
    const PACKAGE: &'static str = "ibc.applications.transfer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.transfer.v1.{}", Self::NAME)
    }
}
/// Params defines the set of IBC transfer parameters.
/// NOTE: To prevent a single token from being transferred, set the
/// TransfersEnabled parameter to true and then set the bank module's SendEnabled
/// parameter for the denomination to false.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// send_enabled enables or disables all cross-chain token transfers from this
    /// chain.
    #[prost(bool, tag = "1")]
    pub send_enabled: bool,
    /// receive_enabled enables or disables all cross-chain token transfers to this
    /// chain.
    #[prost(bool, tag = "2")]
    pub receive_enabled: bool,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "ibc.applications.transfer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.transfer.v1.{}", Self::NAME)
    }
}
/// GenesisState defines the ibc-transfer genesis state
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub denom_traces: ::prost::alloc::vec::Vec<DenomTrace>,
    #[prost(message, optional, tag = "3")]
    pub params: ::core::option::Option<Params>,
    /// total_escrowed contains the total amount of tokens escrowed
    /// by the transfer module
    #[prost(message, repeated, tag = "4")]
    pub total_escrowed:
        ::prost::alloc::vec::Vec<super::super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "ibc.applications.transfer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.transfer.v1.{}", Self::NAME)
    }
}
/// QueryDenomTraceRequest is the request type for the Query/DenomTrace RPC
/// method
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDenomTraceRequest {
    /// hash (in hex format) or denom (full denom with ibc prefix) of the denomination trace information.
    #[prost(string, tag = "1")]
    pub hash: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryDenomTraceRequest {
    const NAME: &'static str = "QueryDenomTraceRequest";
    const PACKAGE: &'static str = "ibc.applications.transfer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.transfer.v1.{}", Self::NAME)
    }
}
/// QueryDenomTraceResponse is the response type for the Query/DenomTrace RPC
/// method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDenomTraceResponse {
    /// denom_trace returns the requested denomination trace information.
    #[prost(message, optional, tag = "1")]
    pub denom_trace: ::core::option::Option<DenomTrace>,
}
impl ::prost::Name for QueryDenomTraceResponse {
    const NAME: &'static str = "QueryDenomTraceResponse";
    const PACKAGE: &'static str = "ibc.applications.transfer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.transfer.v1.{}", Self::NAME)
    }
}
/// QueryConnectionsRequest is the request type for the Query/DenomTraces RPC
/// method
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDenomTracesRequest {
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "1")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageRequest,
    >,
}
impl ::prost::Name for QueryDenomTracesRequest {
    const NAME: &'static str = "QueryDenomTracesRequest";
    const PACKAGE: &'static str = "ibc.applications.transfer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.transfer.v1.{}", Self::NAME)
    }
}
/// QueryConnectionsResponse is the response type for the Query/DenomTraces RPC
/// method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDenomTracesResponse {
    /// denom_traces returns all denominations trace information.
    #[prost(message, repeated, tag = "1")]
    pub denom_traces: ::prost::alloc::vec::Vec<DenomTrace>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageResponse,
    >,
}
impl ::prost::Name for QueryDenomTracesResponse {
    const NAME: &'static str = "QueryDenomTracesResponse";
    const PACKAGE: &'static str = "ibc.applications.transfer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.transfer.v1.{}", Self::NAME)
    }
}
/// QueryParamsRequest is the request type for the Query/Params RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "ibc.applications.transfer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.transfer.v1.{}", Self::NAME)
    }
}
/// QueryParamsResponse is the response type for the Query/Params RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// params defines the parameters of the module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for QueryParamsResponse {
    const NAME: &'static str = "QueryParamsResponse";
    const PACKAGE: &'static str = "ibc.applications.transfer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.transfer.v1.{}", Self::NAME)
    }
}
/// QueryDenomHashRequest is the request type for the Query/DenomHash RPC
/// method
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDenomHashRequest {
    /// The denomination trace (\[port_id\]/[channel_id])+/\[denom\]
    #[prost(string, tag = "1")]
    pub trace: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryDenomHashRequest {
    const NAME: &'static str = "QueryDenomHashRequest";
    const PACKAGE: &'static str = "ibc.applications.transfer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.transfer.v1.{}", Self::NAME)
    }
}
/// QueryDenomHashResponse is the response type for the Query/DenomHash RPC
/// method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDenomHashResponse {
    /// hash (in hex format) of the denomination trace information.
    #[prost(string, tag = "1")]
    pub hash: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryDenomHashResponse {
    const NAME: &'static str = "QueryDenomHashResponse";
    const PACKAGE: &'static str = "ibc.applications.transfer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.transfer.v1.{}", Self::NAME)
    }
}
/// QueryEscrowAddressRequest is the request type for the EscrowAddress RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryEscrowAddressRequest {
    /// unique port identifier
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    /// unique channel identifier
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryEscrowAddressRequest {
    const NAME: &'static str = "QueryEscrowAddressRequest";
    const PACKAGE: &'static str = "ibc.applications.transfer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.transfer.v1.{}", Self::NAME)
    }
}
/// QueryEscrowAddressResponse is the response type of the EscrowAddress RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryEscrowAddressResponse {
    /// the escrow account address
    #[prost(string, tag = "1")]
    pub escrow_address: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryEscrowAddressResponse {
    const NAME: &'static str = "QueryEscrowAddressResponse";
    const PACKAGE: &'static str = "ibc.applications.transfer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.transfer.v1.{}", Self::NAME)
    }
}
/// QueryTotalEscrowForDenomRequest is the request type for TotalEscrowForDenom RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTotalEscrowForDenomRequest {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryTotalEscrowForDenomRequest {
    const NAME: &'static str = "QueryTotalEscrowForDenomRequest";
    const PACKAGE: &'static str = "ibc.applications.transfer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.transfer.v1.{}", Self::NAME)
    }
}
/// QueryTotalEscrowForDenomResponse is the response type for TotalEscrowForDenom RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTotalEscrowForDenomResponse {
    #[prost(message, optional, tag = "1")]
    pub amount: ::core::option::Option<super::super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for QueryTotalEscrowForDenomResponse {
    const NAME: &'static str = "QueryTotalEscrowForDenomResponse";
    const PACKAGE: &'static str = "ibc.applications.transfer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.transfer.v1.{}", Self::NAME)
    }
}
/// MsgTransfer defines a msg to transfer fungible tokens (i.e Coins) between
/// ICS20 enabled chains. See ICS Spec here:
/// <https://github.com/cosmos/ibc/tree/master/spec/app/ics-020-fungible-token-transfer#data-structures>
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgTransfer {
    /// the port on which the packet will be sent
    #[prost(string, tag = "1")]
    pub source_port: ::prost::alloc::string::String,
    /// the channel by which the packet will be sent
    #[prost(string, tag = "2")]
    pub source_channel: ::prost::alloc::string::String,
    /// the tokens to be transferred
    #[prost(message, optional, tag = "3")]
    pub token: ::core::option::Option<super::super::super::super::cosmos::base::v1beta1::Coin>,
    /// the sender address
    #[prost(string, tag = "4")]
    pub sender: ::prost::alloc::string::String,
    /// the recipient address on the destination chain
    #[prost(string, tag = "5")]
    pub receiver: ::prost::alloc::string::String,
    /// Timeout height relative to the current block height.
    /// The timeout is disabled when set to 0.
    #[prost(message, optional, tag = "6")]
    pub timeout_height: ::core::option::Option<super::super::super::core::client::v1::Height>,
    /// Timeout timestamp in absolute nanoseconds since unix epoch.
    /// The timeout is disabled when set to 0.
    #[prost(uint64, tag = "7")]
    pub timeout_timestamp: u64,
    /// optional memo
    #[prost(string, tag = "8")]
    pub memo: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgTransfer {
    const NAME: &'static str = "MsgTransfer";
    const PACKAGE: &'static str = "ibc.applications.transfer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.transfer.v1.{}", Self::NAME)
    }
}
/// MsgTransferResponse defines the Msg/Transfer response type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgTransferResponse {
    /// sequence number of the transfer packet sent
    #[prost(uint64, tag = "1")]
    pub sequence: u64,
}
impl ::prost::Name for MsgTransferResponse {
    const NAME: &'static str = "MsgTransferResponse";
    const PACKAGE: &'static str = "ibc.applications.transfer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.transfer.v1.{}", Self::NAME)
    }
}
/// MsgUpdateParams is the Msg/UpdateParams request type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParams {
    /// signer address
    #[prost(string, tag = "1")]
    pub signer: ::prost::alloc::string::String,
    /// params defines the transfer parameters to update.
    ///
    /// NOTE: All parameters must be supplied.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for MsgUpdateParams {
    const NAME: &'static str = "MsgUpdateParams";
    const PACKAGE: &'static str = "ibc.applications.transfer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.transfer.v1.{}", Self::NAME)
    }
}
/// MsgUpdateParamsResponse defines the response structure for executing a
/// MsgUpdateParams message.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParamsResponse {}
impl ::prost::Name for MsgUpdateParamsResponse {
    const NAME: &'static str = "MsgUpdateParamsResponse";
    const PACKAGE: &'static str = "ibc.applications.transfer.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.transfer.v1.{}", Self::NAME)
    }
}
include!("ibc.applications.transfer.v1.tonic.rs");
// @@protoc_insertion_point(module)
