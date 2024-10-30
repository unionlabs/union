// @generated
/// IncentivizedAcknowledgement is the acknowledgement format to be used by applications wrapped in the fee middleware
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IncentivizedAcknowledgement {
    /// the underlying app acknowledgement bytes
    #[prost(bytes = "vec", tag = "1")]
    pub app_acknowledgement: ::prost::alloc::vec::Vec<u8>,
    /// the relayer address which submits the recv packet message
    #[prost(string, tag = "2")]
    pub forward_relayer_address: ::prost::alloc::string::String,
    /// success flag of the base application callback
    #[prost(bool, tag = "3")]
    pub underlying_app_success: bool,
}
impl ::prost::Name for IncentivizedAcknowledgement {
    const NAME: &'static str = "IncentivizedAcknowledgement";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// Fee defines the ICS29 receive, acknowledgement and timeout fees
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fee {
    /// the packet receive fee
    #[prost(message, repeated, tag = "1")]
    pub recv_fee: ::prost::alloc::vec::Vec<super::super::super::super::cosmos::base::v1beta1::Coin>,
    /// the packet acknowledgement fee
    #[prost(message, repeated, tag = "2")]
    pub ack_fee: ::prost::alloc::vec::Vec<super::super::super::super::cosmos::base::v1beta1::Coin>,
    /// the packet timeout fee
    #[prost(message, repeated, tag = "3")]
    pub timeout_fee:
        ::prost::alloc::vec::Vec<super::super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for Fee {
    const NAME: &'static str = "Fee";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// PacketFee contains ICS29 relayer fees, refund address and optional list of permitted relayers
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PacketFee {
    /// fee encapsulates the recv, ack and timeout fees associated with an IBC packet
    #[prost(message, optional, tag = "1")]
    pub fee: ::core::option::Option<Fee>,
    /// the refund address for unspent fees
    #[prost(string, tag = "2")]
    pub refund_address: ::prost::alloc::string::String,
    /// optional list of relayers permitted to receive fees
    #[prost(string, repeated, tag = "3")]
    pub relayers: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for PacketFee {
    const NAME: &'static str = "PacketFee";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// PacketFees contains a list of type PacketFee
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PacketFees {
    /// list of packet fees
    #[prost(message, repeated, tag = "1")]
    pub packet_fees: ::prost::alloc::vec::Vec<PacketFee>,
}
impl ::prost::Name for PacketFees {
    const NAME: &'static str = "PacketFees";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// IdentifiedPacketFees contains a list of type PacketFee and associated PacketId
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IdentifiedPacketFees {
    /// unique packet identifier comprised of the channel ID, port ID and sequence
    #[prost(message, optional, tag = "1")]
    pub packet_id: ::core::option::Option<super::super::super::core::channel::v1::PacketId>,
    /// list of packet fees
    #[prost(message, repeated, tag = "2")]
    pub packet_fees: ::prost::alloc::vec::Vec<PacketFee>,
}
impl ::prost::Name for IdentifiedPacketFees {
    const NAME: &'static str = "IdentifiedPacketFees";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// GenesisState defines the ICS29 fee middleware genesis state
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// list of identified packet fees
    #[prost(message, repeated, tag = "1")]
    pub identified_fees: ::prost::alloc::vec::Vec<IdentifiedPacketFees>,
    /// list of fee enabled channels
    #[prost(message, repeated, tag = "2")]
    pub fee_enabled_channels: ::prost::alloc::vec::Vec<FeeEnabledChannel>,
    /// list of registered payees
    #[prost(message, repeated, tag = "3")]
    pub registered_payees: ::prost::alloc::vec::Vec<RegisteredPayee>,
    /// list of registered counterparty payees
    #[prost(message, repeated, tag = "4")]
    pub registered_counterparty_payees: ::prost::alloc::vec::Vec<RegisteredCounterpartyPayee>,
    /// list of forward relayer addresses
    #[prost(message, repeated, tag = "5")]
    pub forward_relayers: ::prost::alloc::vec::Vec<ForwardRelayerAddress>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// FeeEnabledChannel contains the PortID & ChannelID for a fee enabled channel
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FeeEnabledChannel {
    /// unique port identifier
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    /// unique channel identifier
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
}
impl ::prost::Name for FeeEnabledChannel {
    const NAME: &'static str = "FeeEnabledChannel";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// RegisteredPayee contains the relayer address and payee address for a specific channel
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RegisteredPayee {
    /// unique channel identifier
    #[prost(string, tag = "1")]
    pub channel_id: ::prost::alloc::string::String,
    /// the relayer address
    #[prost(string, tag = "2")]
    pub relayer: ::prost::alloc::string::String,
    /// the payee address
    #[prost(string, tag = "3")]
    pub payee: ::prost::alloc::string::String,
}
impl ::prost::Name for RegisteredPayee {
    const NAME: &'static str = "RegisteredPayee";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// RegisteredCounterpartyPayee contains the relayer address and counterparty payee address for a specific channel (used
/// for recv fee distribution)
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RegisteredCounterpartyPayee {
    /// unique channel identifier
    #[prost(string, tag = "1")]
    pub channel_id: ::prost::alloc::string::String,
    /// the relayer address
    #[prost(string, tag = "2")]
    pub relayer: ::prost::alloc::string::String,
    /// the counterparty payee address
    #[prost(string, tag = "3")]
    pub counterparty_payee: ::prost::alloc::string::String,
}
impl ::prost::Name for RegisteredCounterpartyPayee {
    const NAME: &'static str = "RegisteredCounterpartyPayee";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// ForwardRelayerAddress contains the forward relayer address and PacketId used for async acknowledgements
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ForwardRelayerAddress {
    /// the forward relayer address
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    /// unique packet identifer comprised of the channel ID, port ID and sequence
    #[prost(message, optional, tag = "2")]
    pub packet_id: ::core::option::Option<super::super::super::core::channel::v1::PacketId>,
}
impl ::prost::Name for ForwardRelayerAddress {
    const NAME: &'static str = "ForwardRelayerAddress";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// QueryIncentivizedPacketsRequest defines the request type for the IncentivizedPackets rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryIncentivizedPacketsRequest {
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "1")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageRequest,
    >,
    /// block height at which to query
    #[prost(uint64, tag = "2")]
    pub query_height: u64,
}
impl ::prost::Name for QueryIncentivizedPacketsRequest {
    const NAME: &'static str = "QueryIncentivizedPacketsRequest";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// QueryIncentivizedPacketsResponse defines the response type for the IncentivizedPackets rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryIncentivizedPacketsResponse {
    /// list of identified fees for incentivized packets
    #[prost(message, repeated, tag = "1")]
    pub incentivized_packets: ::prost::alloc::vec::Vec<IdentifiedPacketFees>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageResponse,
    >,
}
impl ::prost::Name for QueryIncentivizedPacketsResponse {
    const NAME: &'static str = "QueryIncentivizedPacketsResponse";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// QueryIncentivizedPacketRequest defines the request type for the IncentivizedPacket rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryIncentivizedPacketRequest {
    /// unique packet identifier comprised of channel ID, port ID and sequence
    #[prost(message, optional, tag = "1")]
    pub packet_id: ::core::option::Option<super::super::super::core::channel::v1::PacketId>,
    /// block height at which to query
    #[prost(uint64, tag = "2")]
    pub query_height: u64,
}
impl ::prost::Name for QueryIncentivizedPacketRequest {
    const NAME: &'static str = "QueryIncentivizedPacketRequest";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// QueryIncentivizedPacketsResponse defines the response type for the IncentivizedPacket rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryIncentivizedPacketResponse {
    /// the identified fees for the incentivized packet
    #[prost(message, optional, tag = "1")]
    pub incentivized_packet: ::core::option::Option<IdentifiedPacketFees>,
}
impl ::prost::Name for QueryIncentivizedPacketResponse {
    const NAME: &'static str = "QueryIncentivizedPacketResponse";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// QueryIncentivizedPacketsForChannelRequest defines the request type for querying for all incentivized packets
/// for a specific channel
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryIncentivizedPacketsForChannelRequest {
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "1")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageRequest,
    >,
    #[prost(string, tag = "2")]
    pub port_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub channel_id: ::prost::alloc::string::String,
    /// Height to query at
    #[prost(uint64, tag = "4")]
    pub query_height: u64,
}
impl ::prost::Name for QueryIncentivizedPacketsForChannelRequest {
    const NAME: &'static str = "QueryIncentivizedPacketsForChannelRequest";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// QueryIncentivizedPacketsResponse defines the response type for the incentivized packets RPC
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryIncentivizedPacketsForChannelResponse {
    /// Map of all incentivized_packets
    #[prost(message, repeated, tag = "1")]
    pub incentivized_packets: ::prost::alloc::vec::Vec<IdentifiedPacketFees>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageResponse,
    >,
}
impl ::prost::Name for QueryIncentivizedPacketsForChannelResponse {
    const NAME: &'static str = "QueryIncentivizedPacketsForChannelResponse";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// QueryTotalRecvFeesRequest defines the request type for the TotalRecvFees rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTotalRecvFeesRequest {
    /// the packet identifier for the associated fees
    #[prost(message, optional, tag = "1")]
    pub packet_id: ::core::option::Option<super::super::super::core::channel::v1::PacketId>,
}
impl ::prost::Name for QueryTotalRecvFeesRequest {
    const NAME: &'static str = "QueryTotalRecvFeesRequest";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// QueryTotalRecvFeesResponse defines the response type for the TotalRecvFees rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTotalRecvFeesResponse {
    /// the total packet receive fees
    #[prost(message, repeated, tag = "1")]
    pub recv_fees:
        ::prost::alloc::vec::Vec<super::super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for QueryTotalRecvFeesResponse {
    const NAME: &'static str = "QueryTotalRecvFeesResponse";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// QueryTotalAckFeesRequest defines the request type for the TotalAckFees rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTotalAckFeesRequest {
    /// the packet identifier for the associated fees
    #[prost(message, optional, tag = "1")]
    pub packet_id: ::core::option::Option<super::super::super::core::channel::v1::PacketId>,
}
impl ::prost::Name for QueryTotalAckFeesRequest {
    const NAME: &'static str = "QueryTotalAckFeesRequest";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// QueryTotalAckFeesResponse defines the response type for the TotalAckFees rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTotalAckFeesResponse {
    /// the total packet acknowledgement fees
    #[prost(message, repeated, tag = "1")]
    pub ack_fees: ::prost::alloc::vec::Vec<super::super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for QueryTotalAckFeesResponse {
    const NAME: &'static str = "QueryTotalAckFeesResponse";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// QueryTotalTimeoutFeesRequest defines the request type for the TotalTimeoutFees rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTotalTimeoutFeesRequest {
    /// the packet identifier for the associated fees
    #[prost(message, optional, tag = "1")]
    pub packet_id: ::core::option::Option<super::super::super::core::channel::v1::PacketId>,
}
impl ::prost::Name for QueryTotalTimeoutFeesRequest {
    const NAME: &'static str = "QueryTotalTimeoutFeesRequest";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// QueryTotalTimeoutFeesResponse defines the response type for the TotalTimeoutFees rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTotalTimeoutFeesResponse {
    /// the total packet timeout fees
    #[prost(message, repeated, tag = "1")]
    pub timeout_fees:
        ::prost::alloc::vec::Vec<super::super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for QueryTotalTimeoutFeesResponse {
    const NAME: &'static str = "QueryTotalTimeoutFeesResponse";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// QueryPayeeRequest defines the request type for the Payee rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryPayeeRequest {
    /// unique channel identifier
    #[prost(string, tag = "1")]
    pub channel_id: ::prost::alloc::string::String,
    /// the relayer address to which the distribution address is registered
    #[prost(string, tag = "2")]
    pub relayer: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryPayeeRequest {
    const NAME: &'static str = "QueryPayeeRequest";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// QueryPayeeResponse defines the response type for the Payee rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryPayeeResponse {
    /// the payee address to which packet fees are paid out
    #[prost(string, tag = "1")]
    pub payee_address: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryPayeeResponse {
    const NAME: &'static str = "QueryPayeeResponse";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// QueryCounterpartyPayeeRequest defines the request type for the CounterpartyPayee rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCounterpartyPayeeRequest {
    /// unique channel identifier
    #[prost(string, tag = "1")]
    pub channel_id: ::prost::alloc::string::String,
    /// the relayer address to which the counterparty is registered
    #[prost(string, tag = "2")]
    pub relayer: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryCounterpartyPayeeRequest {
    const NAME: &'static str = "QueryCounterpartyPayeeRequest";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// QueryCounterpartyPayeeResponse defines the response type for the CounterpartyPayee rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCounterpartyPayeeResponse {
    /// the counterparty payee address used to compensate forward relaying
    #[prost(string, tag = "1")]
    pub counterparty_payee: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryCounterpartyPayeeResponse {
    const NAME: &'static str = "QueryCounterpartyPayeeResponse";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// QueryFeeEnabledChannelsRequest defines the request type for the FeeEnabledChannels rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryFeeEnabledChannelsRequest {
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "1")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageRequest,
    >,
    /// block height at which to query
    #[prost(uint64, tag = "2")]
    pub query_height: u64,
}
impl ::prost::Name for QueryFeeEnabledChannelsRequest {
    const NAME: &'static str = "QueryFeeEnabledChannelsRequest";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// QueryFeeEnabledChannelsResponse defines the response type for the FeeEnabledChannels rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryFeeEnabledChannelsResponse {
    /// list of fee enabled channels
    #[prost(message, repeated, tag = "1")]
    pub fee_enabled_channels: ::prost::alloc::vec::Vec<FeeEnabledChannel>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageResponse,
    >,
}
impl ::prost::Name for QueryFeeEnabledChannelsResponse {
    const NAME: &'static str = "QueryFeeEnabledChannelsResponse";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// QueryFeeEnabledChannelRequest defines the request type for the FeeEnabledChannel rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryFeeEnabledChannelRequest {
    /// unique port identifier
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    /// unique channel identifier
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryFeeEnabledChannelRequest {
    const NAME: &'static str = "QueryFeeEnabledChannelRequest";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// QueryFeeEnabledChannelResponse defines the response type for the FeeEnabledChannel rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryFeeEnabledChannelResponse {
    /// boolean flag representing the fee enabled channel status
    #[prost(bool, tag = "1")]
    pub fee_enabled: bool,
}
impl ::prost::Name for QueryFeeEnabledChannelResponse {
    const NAME: &'static str = "QueryFeeEnabledChannelResponse";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// MsgRegisterPayee defines the request type for the RegisterPayee rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRegisterPayee {
    /// unique port identifier
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    /// unique channel identifier
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    /// the relayer address
    #[prost(string, tag = "3")]
    pub relayer: ::prost::alloc::string::String,
    /// the payee address
    #[prost(string, tag = "4")]
    pub payee: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgRegisterPayee {
    const NAME: &'static str = "MsgRegisterPayee";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// MsgRegisterPayeeResponse defines the response type for the RegisterPayee rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRegisterPayeeResponse {}
impl ::prost::Name for MsgRegisterPayeeResponse {
    const NAME: &'static str = "MsgRegisterPayeeResponse";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// MsgRegisterCounterpartyPayee defines the request type for the RegisterCounterpartyPayee rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRegisterCounterpartyPayee {
    /// unique port identifier
    #[prost(string, tag = "1")]
    pub port_id: ::prost::alloc::string::String,
    /// unique channel identifier
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    /// the relayer address
    #[prost(string, tag = "3")]
    pub relayer: ::prost::alloc::string::String,
    /// the counterparty payee address
    #[prost(string, tag = "4")]
    pub counterparty_payee: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgRegisterCounterpartyPayee {
    const NAME: &'static str = "MsgRegisterCounterpartyPayee";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// MsgRegisterCounterpartyPayeeResponse defines the response type for the RegisterCounterpartyPayee rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRegisterCounterpartyPayeeResponse {}
impl ::prost::Name for MsgRegisterCounterpartyPayeeResponse {
    const NAME: &'static str = "MsgRegisterCounterpartyPayeeResponse";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// MsgPayPacketFee defines the request type for the PayPacketFee rpc
/// This Msg can be used to pay for a packet at the next sequence send & should be combined with the Msg that will be
/// paid for
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgPayPacketFee {
    /// fee encapsulates the recv, ack and timeout fees associated with an IBC packet
    #[prost(message, optional, tag = "1")]
    pub fee: ::core::option::Option<Fee>,
    /// the source port unique identifier
    #[prost(string, tag = "2")]
    pub source_port_id: ::prost::alloc::string::String,
    /// the source channel unique identifer
    #[prost(string, tag = "3")]
    pub source_channel_id: ::prost::alloc::string::String,
    /// account address to refund fee if necessary
    #[prost(string, tag = "4")]
    pub signer: ::prost::alloc::string::String,
    /// optional list of relayers permitted to the receive packet fees
    #[prost(string, repeated, tag = "5")]
    pub relayers: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for MsgPayPacketFee {
    const NAME: &'static str = "MsgPayPacketFee";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// MsgPayPacketFeeResponse defines the response type for the PayPacketFee rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgPayPacketFeeResponse {}
impl ::prost::Name for MsgPayPacketFeeResponse {
    const NAME: &'static str = "MsgPayPacketFeeResponse";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// MsgPayPacketFeeAsync defines the request type for the PayPacketFeeAsync rpc
/// This Msg can be used to pay for a packet at a specified sequence (instead of the next sequence send)
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgPayPacketFeeAsync {
    /// unique packet identifier comprised of the channel ID, port ID and sequence
    #[prost(message, optional, tag = "1")]
    pub packet_id: ::core::option::Option<super::super::super::core::channel::v1::PacketId>,
    /// the packet fee associated with a particular IBC packet
    #[prost(message, optional, tag = "2")]
    pub packet_fee: ::core::option::Option<PacketFee>,
}
impl ::prost::Name for MsgPayPacketFeeAsync {
    const NAME: &'static str = "MsgPayPacketFeeAsync";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// MsgPayPacketFeeAsyncResponse defines the response type for the PayPacketFeeAsync rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgPayPacketFeeAsyncResponse {}
impl ::prost::Name for MsgPayPacketFeeAsyncResponse {
    const NAME: &'static str = "MsgPayPacketFeeAsyncResponse";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
/// Metadata defines the ICS29 channel specific metadata encoded into the channel version bytestring
/// See ICS004: <https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#Versioning>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Metadata {
    /// fee_version defines the ICS29 fee version
    #[prost(string, tag = "1")]
    pub fee_version: ::prost::alloc::string::String,
    /// app_version defines the underlying application version, which may or may not be a JSON encoded bytestring
    #[prost(string, tag = "2")]
    pub app_version: ::prost::alloc::string::String,
}
impl ::prost::Name for Metadata {
    const NAME: &'static str = "Metadata";
    const PACKAGE: &'static str = "ibc.applications.fee.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.applications.fee.v1.{}", Self::NAME)
    }
}
include!("ibc.applications.fee.v1.tonic.rs");
// @@protoc_insertion_point(module)
