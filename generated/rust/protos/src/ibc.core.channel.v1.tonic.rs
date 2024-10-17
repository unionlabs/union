// @generated
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod query_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::{http::Uri, *};
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
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
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
        pub async fn channel(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryChannelRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryChannelResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/ibc.core.channel.v1.Query/Channel");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.channel.v1.Query", "Channel"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn channels(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryChannelsRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryChannelsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/ibc.core.channel.v1.Query/Channels");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.channel.v1.Query", "Channels"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn connection_channels(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryConnectionChannelsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryConnectionChannelsResponse>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Query/ConnectionChannels",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Query",
                "ConnectionChannels",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn channel_client_state(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryChannelClientStateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryChannelClientStateResponse>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Query/ChannelClientState",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Query",
                "ChannelClientState",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn channel_consensus_state(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryChannelConsensusStateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryChannelConsensusStateResponse>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Query/ChannelConsensusState",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Query",
                "ChannelConsensusState",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn packet_commitment(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryPacketCommitmentRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryPacketCommitmentResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/ibc.core.channel.v1.Query/PacketCommitment");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Query",
                "PacketCommitment",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn packet_commitments(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryPacketCommitmentsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryPacketCommitmentsResponse>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Query/PacketCommitments",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Query",
                "PacketCommitments",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn packet_receipt(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryPacketReceiptRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryPacketReceiptResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/ibc.core.channel.v1.Query/PacketReceipt");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Query",
                "PacketReceipt",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn packet_acknowledgement(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryPacketAcknowledgementRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryPacketAcknowledgementResponse>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Query/PacketAcknowledgement",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Query",
                "PacketAcknowledgement",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn packet_acknowledgements(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryPacketAcknowledgementsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryPacketAcknowledgementsResponse>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Query/PacketAcknowledgements",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Query",
                "PacketAcknowledgements",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn unreceived_packets(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryUnreceivedPacketsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryUnreceivedPacketsResponse>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Query/UnreceivedPackets",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Query",
                "UnreceivedPackets",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn unreceived_acks(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryUnreceivedAcksRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryUnreceivedAcksResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/ibc.core.channel.v1.Query/UnreceivedAcks");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Query",
                "UnreceivedAcks",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn next_sequence_receive(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryNextSequenceReceiveRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryNextSequenceReceiveResponse>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Query/NextSequenceReceive",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Query",
                "NextSequenceReceive",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn next_sequence_send(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryNextSequenceSendRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryNextSequenceSendResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/ibc.core.channel.v1.Query/NextSequenceSend");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Query",
                "NextSequenceSend",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn upgrade_error(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryUpgradeErrorRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryUpgradeErrorResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/ibc.core.channel.v1.Query/UpgradeError");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.channel.v1.Query", "UpgradeError"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn upgrade(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryUpgradeRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryUpgradeResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/ibc.core.channel.v1.Query/Upgrade");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.channel.v1.Query", "Upgrade"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn channel_params(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryChannelParamsRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryChannelParamsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/ibc.core.channel.v1.Query/ChannelParams");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Query",
                "ChannelParams",
            ));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod msg_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::{http::Uri, *};
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
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> MsgClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
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
        pub async fn channel_open_init(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgChannelOpenInit>,
        ) -> std::result::Result<tonic::Response<super::MsgChannelOpenInitResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/ibc.core.channel.v1.Msg/ChannelOpenInit");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Msg",
                "ChannelOpenInit",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn channel_open_try(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgChannelOpenTry>,
        ) -> std::result::Result<tonic::Response<super::MsgChannelOpenTryResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/ibc.core.channel.v1.Msg/ChannelOpenTry");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.channel.v1.Msg", "ChannelOpenTry"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn channel_open_ack(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgChannelOpenAck>,
        ) -> std::result::Result<tonic::Response<super::MsgChannelOpenAckResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/ibc.core.channel.v1.Msg/ChannelOpenAck");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.channel.v1.Msg", "ChannelOpenAck"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn channel_open_confirm(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgChannelOpenConfirm>,
        ) -> std::result::Result<tonic::Response<super::MsgChannelOpenConfirmResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/ibc.core.channel.v1.Msg/ChannelOpenConfirm");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Msg",
                "ChannelOpenConfirm",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn channel_close_init(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgChannelCloseInit>,
        ) -> std::result::Result<tonic::Response<super::MsgChannelCloseInitResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/ibc.core.channel.v1.Msg/ChannelCloseInit");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Msg",
                "ChannelCloseInit",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn channel_close_confirm(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgChannelCloseConfirm>,
        ) -> std::result::Result<
            tonic::Response<super::MsgChannelCloseConfirmResponse>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Msg/ChannelCloseConfirm",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Msg",
                "ChannelCloseConfirm",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn recv_packet(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgRecvPacket>,
        ) -> std::result::Result<tonic::Response<super::MsgRecvPacketResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/ibc.core.channel.v1.Msg/RecvPacket");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.channel.v1.Msg", "RecvPacket"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn timeout(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgTimeout>,
        ) -> std::result::Result<tonic::Response<super::MsgTimeoutResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/ibc.core.channel.v1.Msg/Timeout");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.channel.v1.Msg", "Timeout"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn timeout_on_close(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgTimeoutOnClose>,
        ) -> std::result::Result<tonic::Response<super::MsgTimeoutOnCloseResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/ibc.core.channel.v1.Msg/TimeoutOnClose");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.channel.v1.Msg", "TimeoutOnClose"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn acknowledgement(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgAcknowledgement>,
        ) -> std::result::Result<tonic::Response<super::MsgAcknowledgementResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/ibc.core.channel.v1.Msg/Acknowledgement");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Msg",
                "Acknowledgement",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn channel_upgrade_init(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgChannelUpgradeInit>,
        ) -> std::result::Result<tonic::Response<super::MsgChannelUpgradeInitResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/ibc.core.channel.v1.Msg/ChannelUpgradeInit");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Msg",
                "ChannelUpgradeInit",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn channel_upgrade_try(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgChannelUpgradeTry>,
        ) -> std::result::Result<tonic::Response<super::MsgChannelUpgradeTryResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/ibc.core.channel.v1.Msg/ChannelUpgradeTry");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Msg",
                "ChannelUpgradeTry",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn channel_upgrade_ack(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgChannelUpgradeAck>,
        ) -> std::result::Result<tonic::Response<super::MsgChannelUpgradeAckResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/ibc.core.channel.v1.Msg/ChannelUpgradeAck");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Msg",
                "ChannelUpgradeAck",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn channel_upgrade_confirm(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgChannelUpgradeConfirm>,
        ) -> std::result::Result<
            tonic::Response<super::MsgChannelUpgradeConfirmResponse>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Msg/ChannelUpgradeConfirm",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Msg",
                "ChannelUpgradeConfirm",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn channel_upgrade_open(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgChannelUpgradeOpen>,
        ) -> std::result::Result<tonic::Response<super::MsgChannelUpgradeOpenResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/ibc.core.channel.v1.Msg/ChannelUpgradeOpen");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Msg",
                "ChannelUpgradeOpen",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn channel_upgrade_timeout(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgChannelUpgradeTimeout>,
        ) -> std::result::Result<
            tonic::Response<super::MsgChannelUpgradeTimeoutResponse>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Msg/ChannelUpgradeTimeout",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Msg",
                "ChannelUpgradeTimeout",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn channel_upgrade_cancel(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgChannelUpgradeCancel>,
        ) -> std::result::Result<
            tonic::Response<super::MsgChannelUpgradeCancelResponse>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Msg/ChannelUpgradeCancel",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Msg",
                "ChannelUpgradeCancel",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_channel_params(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgUpdateParams>,
        ) -> std::result::Result<tonic::Response<super::MsgUpdateParamsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Msg/UpdateChannelParams",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Msg",
                "UpdateChannelParams",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn prune_acknowledgements(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgPruneAcknowledgements>,
        ) -> std::result::Result<
            tonic::Response<super::MsgPruneAcknowledgementsResponse>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.core.channel.v1.Msg/PruneAcknowledgements",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "ibc.core.channel.v1.Msg",
                "PruneAcknowledgements",
            ));
            self.inner.unary(req, path, codec).await
        }
    }
}
