/// Fee defines the ICS29 receive, acknowledgement and timeout fees
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fee {
    /// the packet receive fee
    #[prost(message, repeated, tag = "1")]
    pub recv_fee: ::prost::alloc::vec::Vec<
        super::super::super::super::cosmos::base::v1beta1::Coin,
    >,
    /// the packet acknowledgement fee
    #[prost(message, repeated, tag = "2")]
    pub ack_fee: ::prost::alloc::vec::Vec<
        super::super::super::super::cosmos::base::v1beta1::Coin,
    >,
    /// the packet timeout fee
    #[prost(message, repeated, tag = "3")]
    pub timeout_fee: ::prost::alloc::vec::Vec<
        super::super::super::super::cosmos::base::v1beta1::Coin,
    >,
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
/// PacketFees contains a list of type PacketFee
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PacketFees {
    /// list of packet fees
    #[prost(message, repeated, tag = "1")]
    pub packet_fees: ::prost::alloc::vec::Vec<PacketFee>,
}
/// IdentifiedPacketFees contains a list of type PacketFee and associated PacketId
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IdentifiedPacketFees {
    /// unique packet identifier comprised of the channel ID, port ID and sequence
    #[prost(message, optional, tag = "1")]
    pub packet_id: ::core::option::Option<
        super::super::super::core::channel::v1::PacketId,
    >,
    /// list of packet fees
    #[prost(message, repeated, tag = "2")]
    pub packet_fees: ::prost::alloc::vec::Vec<PacketFee>,
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
    pub registered_counterparty_payees: ::prost::alloc::vec::Vec<
        RegisteredCounterpartyPayee,
    >,
    /// list of forward relayer addresses
    #[prost(message, repeated, tag = "5")]
    pub forward_relayers: ::prost::alloc::vec::Vec<ForwardRelayerAddress>,
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
/// ForwardRelayerAddress contains the forward relayer address and PacketId used for async acknowledgements
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ForwardRelayerAddress {
    /// the forward relayer address
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    /// unique packet identifer comprised of the channel ID, port ID and sequence
    #[prost(message, optional, tag = "2")]
    pub packet_id: ::core::option::Option<
        super::super::super::core::channel::v1::PacketId,
    >,
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
/// QueryIncentivizedPacketRequest defines the request type for the IncentivizedPacket rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryIncentivizedPacketRequest {
    /// unique packet identifier comprised of channel ID, port ID and sequence
    #[prost(message, optional, tag = "1")]
    pub packet_id: ::core::option::Option<
        super::super::super::core::channel::v1::PacketId,
    >,
    /// block height at which to query
    #[prost(uint64, tag = "2")]
    pub query_height: u64,
}
/// QueryIncentivizedPacketsResponse defines the response type for the IncentivizedPacket rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryIncentivizedPacketResponse {
    /// the identified fees for the incentivized packet
    #[prost(message, optional, tag = "1")]
    pub incentivized_packet: ::core::option::Option<IdentifiedPacketFees>,
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
/// QueryTotalRecvFeesRequest defines the request type for the TotalRecvFees rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTotalRecvFeesRequest {
    /// the packet identifier for the associated fees
    #[prost(message, optional, tag = "1")]
    pub packet_id: ::core::option::Option<
        super::super::super::core::channel::v1::PacketId,
    >,
}
/// QueryTotalRecvFeesResponse defines the response type for the TotalRecvFees rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTotalRecvFeesResponse {
    /// the total packet receive fees
    #[prost(message, repeated, tag = "1")]
    pub recv_fees: ::prost::alloc::vec::Vec<
        super::super::super::super::cosmos::base::v1beta1::Coin,
    >,
}
/// QueryTotalAckFeesRequest defines the request type for the TotalAckFees rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTotalAckFeesRequest {
    /// the packet identifier for the associated fees
    #[prost(message, optional, tag = "1")]
    pub packet_id: ::core::option::Option<
        super::super::super::core::channel::v1::PacketId,
    >,
}
/// QueryTotalAckFeesResponse defines the response type for the TotalAckFees rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTotalAckFeesResponse {
    /// the total packet acknowledgement fees
    #[prost(message, repeated, tag = "1")]
    pub ack_fees: ::prost::alloc::vec::Vec<
        super::super::super::super::cosmos::base::v1beta1::Coin,
    >,
}
/// QueryTotalTimeoutFeesRequest defines the request type for the TotalTimeoutFees rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTotalTimeoutFeesRequest {
    /// the packet identifier for the associated fees
    #[prost(message, optional, tag = "1")]
    pub packet_id: ::core::option::Option<
        super::super::super::core::channel::v1::PacketId,
    >,
}
/// QueryTotalTimeoutFeesResponse defines the response type for the TotalTimeoutFees rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTotalTimeoutFeesResponse {
    /// the total packet timeout fees
    #[prost(message, repeated, tag = "1")]
    pub timeout_fees: ::prost::alloc::vec::Vec<
        super::super::super::super::cosmos::base::v1beta1::Coin,
    >,
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
/// QueryPayeeResponse defines the response type for the Payee rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryPayeeResponse {
    /// the payee address to which packet fees are paid out
    #[prost(string, tag = "1")]
    pub payee_address: ::prost::alloc::string::String,
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
/// QueryCounterpartyPayeeResponse defines the response type for the CounterpartyPayee rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCounterpartyPayeeResponse {
    /// the counterparty payee address used to compensate forward relaying
    #[prost(string, tag = "1")]
    pub counterparty_payee: ::prost::alloc::string::String,
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
/// QueryFeeEnabledChannelResponse defines the response type for the FeeEnabledChannel rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryFeeEnabledChannelResponse {
    /// boolean flag representing the fee enabled channel status
    #[prost(bool, tag = "1")]
    pub fee_enabled: bool,
}
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod query_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Query defines the ICS29 gRPC querier service.
    #[derive(Debug, Clone)]
    pub struct QueryClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl QueryClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> QueryClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> QueryClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            QueryClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// IncentivizedPackets returns all incentivized packets and their associated fees
        pub async fn incentivized_packets(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryIncentivizedPacketsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryIncentivizedPacketsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.fee.v1.Query/IncentivizedPackets",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "ibc.applications.fee.v1.Query",
                        "IncentivizedPackets",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// IncentivizedPacket returns all packet fees for a packet given its identifier
        pub async fn incentivized_packet(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryIncentivizedPacketRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryIncentivizedPacketResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.fee.v1.Query/IncentivizedPacket",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "ibc.applications.fee.v1.Query",
                        "IncentivizedPacket",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Gets all incentivized packets for a specific channel
        pub async fn incentivized_packets_for_channel(
            &mut self,
            request: impl tonic::IntoRequest<
                super::QueryIncentivizedPacketsForChannelRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<super::QueryIncentivizedPacketsForChannelResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.fee.v1.Query/IncentivizedPacketsForChannel",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "ibc.applications.fee.v1.Query",
                        "IncentivizedPacketsForChannel",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// TotalRecvFees returns the total receive fees for a packet given its identifier
        pub async fn total_recv_fees(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryTotalRecvFeesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryTotalRecvFeesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.fee.v1.Query/TotalRecvFees",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("ibc.applications.fee.v1.Query", "TotalRecvFees"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// TotalAckFees returns the total acknowledgement fees for a packet given its identifier
        pub async fn total_ack_fees(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryTotalAckFeesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryTotalAckFeesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.fee.v1.Query/TotalAckFees",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("ibc.applications.fee.v1.Query", "TotalAckFees"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// TotalTimeoutFees returns the total timeout fees for a packet given its identifier
        pub async fn total_timeout_fees(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryTotalTimeoutFeesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryTotalTimeoutFeesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.fee.v1.Query/TotalTimeoutFees",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("ibc.applications.fee.v1.Query", "TotalTimeoutFees"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Payee returns the registered payee address for a specific channel given the relayer address
        pub async fn payee(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryPayeeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryPayeeResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.fee.v1.Query/Payee",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.applications.fee.v1.Query", "Payee"));
            self.inner.unary(req, path, codec).await
        }
        /// CounterpartyPayee returns the registered counterparty payee for forward relaying
        pub async fn counterparty_payee(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryCounterpartyPayeeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryCounterpartyPayeeResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.fee.v1.Query/CounterpartyPayee",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("ibc.applications.fee.v1.Query", "CounterpartyPayee"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// FeeEnabledChannels returns a list of all fee enabled channels
        pub async fn fee_enabled_channels(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryFeeEnabledChannelsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryFeeEnabledChannelsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.fee.v1.Query/FeeEnabledChannels",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "ibc.applications.fee.v1.Query",
                        "FeeEnabledChannels",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// FeeEnabledChannel returns true if the provided port and channel identifiers belong to a fee enabled channel
        pub async fn fee_enabled_channel(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryFeeEnabledChannelRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryFeeEnabledChannelResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.fee.v1.Query/FeeEnabledChannel",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("ibc.applications.fee.v1.Query", "FeeEnabledChannel"),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "server")]
pub mod query_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with QueryServer.
    #[async_trait]
    pub trait Query: Send + Sync + 'static {
        /// IncentivizedPackets returns all incentivized packets and their associated fees
        async fn incentivized_packets(
            &self,
            request: tonic::Request<super::QueryIncentivizedPacketsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryIncentivizedPacketsResponse>,
            tonic::Status,
        >;
        /// IncentivizedPacket returns all packet fees for a packet given its identifier
        async fn incentivized_packet(
            &self,
            request: tonic::Request<super::QueryIncentivizedPacketRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryIncentivizedPacketResponse>,
            tonic::Status,
        >;
        /// Gets all incentivized packets for a specific channel
        async fn incentivized_packets_for_channel(
            &self,
            request: tonic::Request<super::QueryIncentivizedPacketsForChannelRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryIncentivizedPacketsForChannelResponse>,
            tonic::Status,
        >;
        /// TotalRecvFees returns the total receive fees for a packet given its identifier
        async fn total_recv_fees(
            &self,
            request: tonic::Request<super::QueryTotalRecvFeesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryTotalRecvFeesResponse>,
            tonic::Status,
        >;
        /// TotalAckFees returns the total acknowledgement fees for a packet given its identifier
        async fn total_ack_fees(
            &self,
            request: tonic::Request<super::QueryTotalAckFeesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryTotalAckFeesResponse>,
            tonic::Status,
        >;
        /// TotalTimeoutFees returns the total timeout fees for a packet given its identifier
        async fn total_timeout_fees(
            &self,
            request: tonic::Request<super::QueryTotalTimeoutFeesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryTotalTimeoutFeesResponse>,
            tonic::Status,
        >;
        /// Payee returns the registered payee address for a specific channel given the relayer address
        async fn payee(
            &self,
            request: tonic::Request<super::QueryPayeeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryPayeeResponse>,
            tonic::Status,
        >;
        /// CounterpartyPayee returns the registered counterparty payee for forward relaying
        async fn counterparty_payee(
            &self,
            request: tonic::Request<super::QueryCounterpartyPayeeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryCounterpartyPayeeResponse>,
            tonic::Status,
        >;
        /// FeeEnabledChannels returns a list of all fee enabled channels
        async fn fee_enabled_channels(
            &self,
            request: tonic::Request<super::QueryFeeEnabledChannelsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryFeeEnabledChannelsResponse>,
            tonic::Status,
        >;
        /// FeeEnabledChannel returns true if the provided port and channel identifiers belong to a fee enabled channel
        async fn fee_enabled_channel(
            &self,
            request: tonic::Request<super::QueryFeeEnabledChannelRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryFeeEnabledChannelResponse>,
            tonic::Status,
        >;
    }
    /// Query defines the ICS29 gRPC querier service.
    #[derive(Debug)]
    pub struct QueryServer<T: Query> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Query> QueryServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for QueryServer<T>
    where
        T: Query,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/ibc.applications.fee.v1.Query/IncentivizedPackets" => {
                    #[allow(non_camel_case_types)]
                    struct IncentivizedPacketsSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryIncentivizedPacketsRequest>
                    for IncentivizedPacketsSvc<T> {
                        type Response = super::QueryIncentivizedPacketsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::QueryIncentivizedPacketsRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).incentivized_packets(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = IncentivizedPacketsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.applications.fee.v1.Query/IncentivizedPacket" => {
                    #[allow(non_camel_case_types)]
                    struct IncentivizedPacketSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryIncentivizedPacketRequest>
                    for IncentivizedPacketSvc<T> {
                        type Response = super::QueryIncentivizedPacketResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::QueryIncentivizedPacketRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).incentivized_packet(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = IncentivizedPacketSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.applications.fee.v1.Query/IncentivizedPacketsForChannel" => {
                    #[allow(non_camel_case_types)]
                    struct IncentivizedPacketsForChannelSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<
                        super::QueryIncentivizedPacketsForChannelRequest,
                    > for IncentivizedPacketsForChannelSvc<T> {
                        type Response = super::QueryIncentivizedPacketsForChannelResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::QueryIncentivizedPacketsForChannelRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).incentivized_packets_for_channel(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = IncentivizedPacketsForChannelSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.applications.fee.v1.Query/TotalRecvFees" => {
                    #[allow(non_camel_case_types)]
                    struct TotalRecvFeesSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryTotalRecvFeesRequest>
                    for TotalRecvFeesSvc<T> {
                        type Response = super::QueryTotalRecvFeesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryTotalRecvFeesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).total_recv_fees(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TotalRecvFeesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.applications.fee.v1.Query/TotalAckFees" => {
                    #[allow(non_camel_case_types)]
                    struct TotalAckFeesSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryTotalAckFeesRequest>
                    for TotalAckFeesSvc<T> {
                        type Response = super::QueryTotalAckFeesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryTotalAckFeesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).total_ack_fees(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TotalAckFeesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.applications.fee.v1.Query/TotalTimeoutFees" => {
                    #[allow(non_camel_case_types)]
                    struct TotalTimeoutFeesSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryTotalTimeoutFeesRequest>
                    for TotalTimeoutFeesSvc<T> {
                        type Response = super::QueryTotalTimeoutFeesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryTotalTimeoutFeesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).total_timeout_fees(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TotalTimeoutFeesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.applications.fee.v1.Query/Payee" => {
                    #[allow(non_camel_case_types)]
                    struct PayeeSvc<T: Query>(pub Arc<T>);
                    impl<T: Query> tonic::server::UnaryService<super::QueryPayeeRequest>
                    for PayeeSvc<T> {
                        type Response = super::QueryPayeeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryPayeeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).payee(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PayeeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.applications.fee.v1.Query/CounterpartyPayee" => {
                    #[allow(non_camel_case_types)]
                    struct CounterpartyPayeeSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryCounterpartyPayeeRequest>
                    for CounterpartyPayeeSvc<T> {
                        type Response = super::QueryCounterpartyPayeeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryCounterpartyPayeeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).counterparty_payee(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CounterpartyPayeeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.applications.fee.v1.Query/FeeEnabledChannels" => {
                    #[allow(non_camel_case_types)]
                    struct FeeEnabledChannelsSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryFeeEnabledChannelsRequest>
                    for FeeEnabledChannelsSvc<T> {
                        type Response = super::QueryFeeEnabledChannelsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::QueryFeeEnabledChannelsRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).fee_enabled_channels(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = FeeEnabledChannelsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.applications.fee.v1.Query/FeeEnabledChannel" => {
                    #[allow(non_camel_case_types)]
                    struct FeeEnabledChannelSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryFeeEnabledChannelRequest>
                    for FeeEnabledChannelSvc<T> {
                        type Response = super::QueryFeeEnabledChannelResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryFeeEnabledChannelRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).fee_enabled_channel(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = FeeEnabledChannelSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: Query> Clone for QueryServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: Query> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Query> tonic::server::NamedService for QueryServer<T> {
        const NAME: &'static str = "ibc.applications.fee.v1.Query";
    }
}
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
/// MsgRegisterPayeeResponse defines the response type for the RegisterPayee rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRegisterPayeeResponse {}
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
/// MsgRegisterCounterpartyPayeeResponse defines the response type for the RegisterCounterpartyPayee rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRegisterCounterpartyPayeeResponse {}
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
/// MsgPayPacketFeeResponse defines the response type for the PayPacketFee rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgPayPacketFeeResponse {}
/// MsgPayPacketFeeAsync defines the request type for the PayPacketFeeAsync rpc
/// This Msg can be used to pay for a packet at a specified sequence (instead of the next sequence send)
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgPayPacketFeeAsync {
    /// unique packet identifier comprised of the channel ID, port ID and sequence
    #[prost(message, optional, tag = "1")]
    pub packet_id: ::core::option::Option<
        super::super::super::core::channel::v1::PacketId,
    >,
    /// the packet fee associated with a particular IBC packet
    #[prost(message, optional, tag = "2")]
    pub packet_fee: ::core::option::Option<PacketFee>,
}
/// MsgPayPacketFeeAsyncResponse defines the response type for the PayPacketFeeAsync rpc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgPayPacketFeeAsyncResponse {}
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod msg_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Msg defines the ICS29 Msg service.
    #[derive(Debug, Clone)]
    pub struct MsgClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl MsgClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> MsgClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> MsgClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            MsgClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// RegisterPayee defines a rpc handler method for MsgRegisterPayee
        /// RegisterPayee is called by the relayer on each channelEnd and allows them to set an optional
        /// payee to which reverse and timeout relayer packet fees will be paid out. The payee should be registered on
        /// the source chain from which packets originate as this is where fee distribution takes place. This function may be
        /// called more than once by a relayer, in which case, the latest payee is always used.
        pub async fn register_payee(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgRegisterPayee>,
        ) -> std::result::Result<
            tonic::Response<super::MsgRegisterPayeeResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.fee.v1.Msg/RegisterPayee",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.applications.fee.v1.Msg", "RegisterPayee"));
            self.inner.unary(req, path, codec).await
        }
        /// RegisterCounterpartyPayee defines a rpc handler method for MsgRegisterCounterpartyPayee
        /// RegisterCounterpartyPayee is called by the relayer on each channelEnd and allows them to specify the counterparty
        /// payee address before relaying. This ensures they will be properly compensated for forward relaying since
        /// the destination chain must include the registered counterparty payee address in the acknowledgement. This function
        /// may be called more than once by a relayer, in which case, the latest counterparty payee address is always used.
        pub async fn register_counterparty_payee(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgRegisterCounterpartyPayee>,
        ) -> std::result::Result<
            tonic::Response<super::MsgRegisterCounterpartyPayeeResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.fee.v1.Msg/RegisterCounterpartyPayee",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "ibc.applications.fee.v1.Msg",
                        "RegisterCounterpartyPayee",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// PayPacketFee defines a rpc handler method for MsgPayPacketFee
        /// PayPacketFee is an open callback that may be called by any module/user that wishes to escrow funds in order to
        /// incentivize the relaying of the packet at the next sequence
        /// NOTE: This method is intended to be used within a multi msg transaction, where the subsequent msg that follows
        /// initiates the lifecycle of the incentivized packet
        pub async fn pay_packet_fee(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgPayPacketFee>,
        ) -> std::result::Result<
            tonic::Response<super::MsgPayPacketFeeResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.fee.v1.Msg/PayPacketFee",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.applications.fee.v1.Msg", "PayPacketFee"));
            self.inner.unary(req, path, codec).await
        }
        /// PayPacketFeeAsync defines a rpc handler method for MsgPayPacketFeeAsync
        /// PayPacketFeeAsync is an open callback that may be called by any module/user that wishes to escrow funds in order to
        /// incentivize the relaying of a known packet (i.e. at a particular sequence)
        pub async fn pay_packet_fee_async(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgPayPacketFeeAsync>,
        ) -> std::result::Result<
            tonic::Response<super::MsgPayPacketFeeAsyncResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.fee.v1.Msg/PayPacketFeeAsync",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("ibc.applications.fee.v1.Msg", "PayPacketFeeAsync"),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "server")]
pub mod msg_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with MsgServer.
    #[async_trait]
    pub trait Msg: Send + Sync + 'static {
        /// RegisterPayee defines a rpc handler method for MsgRegisterPayee
        /// RegisterPayee is called by the relayer on each channelEnd and allows them to set an optional
        /// payee to which reverse and timeout relayer packet fees will be paid out. The payee should be registered on
        /// the source chain from which packets originate as this is where fee distribution takes place. This function may be
        /// called more than once by a relayer, in which case, the latest payee is always used.
        async fn register_payee(
            &self,
            request: tonic::Request<super::MsgRegisterPayee>,
        ) -> std::result::Result<
            tonic::Response<super::MsgRegisterPayeeResponse>,
            tonic::Status,
        >;
        /// RegisterCounterpartyPayee defines a rpc handler method for MsgRegisterCounterpartyPayee
        /// RegisterCounterpartyPayee is called by the relayer on each channelEnd and allows them to specify the counterparty
        /// payee address before relaying. This ensures they will be properly compensated for forward relaying since
        /// the destination chain must include the registered counterparty payee address in the acknowledgement. This function
        /// may be called more than once by a relayer, in which case, the latest counterparty payee address is always used.
        async fn register_counterparty_payee(
            &self,
            request: tonic::Request<super::MsgRegisterCounterpartyPayee>,
        ) -> std::result::Result<
            tonic::Response<super::MsgRegisterCounterpartyPayeeResponse>,
            tonic::Status,
        >;
        /// PayPacketFee defines a rpc handler method for MsgPayPacketFee
        /// PayPacketFee is an open callback that may be called by any module/user that wishes to escrow funds in order to
        /// incentivize the relaying of the packet at the next sequence
        /// NOTE: This method is intended to be used within a multi msg transaction, where the subsequent msg that follows
        /// initiates the lifecycle of the incentivized packet
        async fn pay_packet_fee(
            &self,
            request: tonic::Request<super::MsgPayPacketFee>,
        ) -> std::result::Result<
            tonic::Response<super::MsgPayPacketFeeResponse>,
            tonic::Status,
        >;
        /// PayPacketFeeAsync defines a rpc handler method for MsgPayPacketFeeAsync
        /// PayPacketFeeAsync is an open callback that may be called by any module/user that wishes to escrow funds in order to
        /// incentivize the relaying of a known packet (i.e. at a particular sequence)
        async fn pay_packet_fee_async(
            &self,
            request: tonic::Request<super::MsgPayPacketFeeAsync>,
        ) -> std::result::Result<
            tonic::Response<super::MsgPayPacketFeeAsyncResponse>,
            tonic::Status,
        >;
    }
    /// Msg defines the ICS29 Msg service.
    #[derive(Debug)]
    pub struct MsgServer<T: Msg> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Msg> MsgServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for MsgServer<T>
    where
        T: Msg,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/ibc.applications.fee.v1.Msg/RegisterPayee" => {
                    #[allow(non_camel_case_types)]
                    struct RegisterPayeeSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgRegisterPayee>
                    for RegisterPayeeSvc<T> {
                        type Response = super::MsgRegisterPayeeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgRegisterPayee>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).register_payee(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = RegisterPayeeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.applications.fee.v1.Msg/RegisterCounterpartyPayee" => {
                    #[allow(non_camel_case_types)]
                    struct RegisterCounterpartyPayeeSvc<T: Msg>(pub Arc<T>);
                    impl<
                        T: Msg,
                    > tonic::server::UnaryService<super::MsgRegisterCounterpartyPayee>
                    for RegisterCounterpartyPayeeSvc<T> {
                        type Response = super::MsgRegisterCounterpartyPayeeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgRegisterCounterpartyPayee>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).register_counterparty_payee(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = RegisterCounterpartyPayeeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.applications.fee.v1.Msg/PayPacketFee" => {
                    #[allow(non_camel_case_types)]
                    struct PayPacketFeeSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgPayPacketFee>
                    for PayPacketFeeSvc<T> {
                        type Response = super::MsgPayPacketFeeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgPayPacketFee>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).pay_packet_fee(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PayPacketFeeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.applications.fee.v1.Msg/PayPacketFeeAsync" => {
                    #[allow(non_camel_case_types)]
                    struct PayPacketFeeAsyncSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgPayPacketFeeAsync>
                    for PayPacketFeeAsyncSvc<T> {
                        type Response = super::MsgPayPacketFeeAsyncResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgPayPacketFeeAsync>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).pay_packet_fee_async(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PayPacketFeeAsyncSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: Msg> Clone for MsgServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: Msg> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Msg> tonic::server::NamedService for MsgServer<T> {
        const NAME: &'static str = "ibc.applications.fee.v1.Msg";
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
