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
            D: std::convert::TryInto<tonic::transport::Endpoint>,
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
        pub async fn incentivized_packets(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryIncentivizedPacketsRequest>,
        ) -> Result<tonic::Response<super::QueryIncentivizedPacketsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.fee.v1.Query/IncentivizedPackets",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn incentivized_packet(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryIncentivizedPacketRequest>,
        ) -> Result<tonic::Response<super::QueryIncentivizedPacketResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.fee.v1.Query/IncentivizedPacket",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn incentivized_packets_for_channel(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryIncentivizedPacketsForChannelRequest>,
        ) -> Result<tonic::Response<super::QueryIncentivizedPacketsForChannelResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.fee.v1.Query/IncentivizedPacketsForChannel",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn total_recv_fees(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryTotalRecvFeesRequest>,
        ) -> Result<tonic::Response<super::QueryTotalRecvFeesResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.fee.v1.Query/TotalRecvFees",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn total_ack_fees(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryTotalAckFeesRequest>,
        ) -> Result<tonic::Response<super::QueryTotalAckFeesResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/ibc.applications.fee.v1.Query/TotalAckFees");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn total_timeout_fees(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryTotalTimeoutFeesRequest>,
        ) -> Result<tonic::Response<super::QueryTotalTimeoutFeesResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.fee.v1.Query/TotalTimeoutFees",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn payee(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryPayeeRequest>,
        ) -> Result<tonic::Response<super::QueryPayeeResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/ibc.applications.fee.v1.Query/Payee");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn counterparty_payee(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryCounterpartyPayeeRequest>,
        ) -> Result<tonic::Response<super::QueryCounterpartyPayeeResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.fee.v1.Query/CounterpartyPayee",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn fee_enabled_channels(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryFeeEnabledChannelsRequest>,
        ) -> Result<tonic::Response<super::QueryFeeEnabledChannelsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.fee.v1.Query/FeeEnabledChannels",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn fee_enabled_channel(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryFeeEnabledChannelRequest>,
        ) -> Result<tonic::Response<super::QueryFeeEnabledChannelResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.fee.v1.Query/FeeEnabledChannel",
            );
            self.inner.unary(request.into_request(), path, codec).await
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
            D: std::convert::TryInto<tonic::transport::Endpoint>,
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
        pub async fn register_payee(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgRegisterPayee>,
        ) -> Result<tonic::Response<super::MsgRegisterPayeeResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/ibc.applications.fee.v1.Msg/RegisterPayee");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn register_counterparty_payee(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgRegisterCounterpartyPayee>,
        ) -> Result<tonic::Response<super::MsgRegisterCounterpartyPayeeResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.fee.v1.Msg/RegisterCounterpartyPayee",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn pay_packet_fee(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgPayPacketFee>,
        ) -> Result<tonic::Response<super::MsgPayPacketFeeResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/ibc.applications.fee.v1.Msg/PayPacketFee");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn pay_packet_fee_async(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgPayPacketFeeAsync>,
        ) -> Result<tonic::Response<super::MsgPayPacketFeeAsyncResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.fee.v1.Msg/PayPacketFeeAsync",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
