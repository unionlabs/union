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
        pub async fn params(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryParamsRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryParamsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/osmosis.protorev.v1beta1.Query/Params");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("osmosis.protorev.v1beta1.Query", "Params"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_proto_rev_number_of_trades(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryGetProtoRevNumberOfTradesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryGetProtoRevNumberOfTradesResponse>,
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
                "/osmosis.protorev.v1beta1.Query/GetProtoRevNumberOfTrades",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.protorev.v1beta1.Query",
                "GetProtoRevNumberOfTrades",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_proto_rev_profits_by_denom(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryGetProtoRevProfitsByDenomRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryGetProtoRevProfitsByDenomResponse>,
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
                "/osmosis.protorev.v1beta1.Query/GetProtoRevProfitsByDenom",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.protorev.v1beta1.Query",
                "GetProtoRevProfitsByDenom",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_proto_rev_all_profits(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryGetProtoRevAllProfitsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryGetProtoRevAllProfitsResponse>,
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
                "/osmosis.protorev.v1beta1.Query/GetProtoRevAllProfits",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.protorev.v1beta1.Query",
                "GetProtoRevAllProfits",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_proto_rev_statistics_by_route(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryGetProtoRevStatisticsByRouteRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryGetProtoRevStatisticsByRouteResponse>,
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
                "/osmosis.protorev.v1beta1.Query/GetProtoRevStatisticsByRoute",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.protorev.v1beta1.Query",
                "GetProtoRevStatisticsByRoute",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_proto_rev_all_route_statistics(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryGetProtoRevAllRouteStatisticsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryGetProtoRevAllRouteStatisticsResponse>,
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
                "/osmosis.protorev.v1beta1.Query/GetProtoRevAllRouteStatistics",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.protorev.v1beta1.Query",
                "GetProtoRevAllRouteStatistics",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_proto_rev_token_pair_arb_routes(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryGetProtoRevTokenPairArbRoutesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryGetProtoRevTokenPairArbRoutesResponse>,
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
                "/osmosis.protorev.v1beta1.Query/GetProtoRevTokenPairArbRoutes",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.protorev.v1beta1.Query",
                "GetProtoRevTokenPairArbRoutes",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_proto_rev_admin_account(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryGetProtoRevAdminAccountRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryGetProtoRevAdminAccountResponse>,
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
                "/osmosis.protorev.v1beta1.Query/GetProtoRevAdminAccount",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.protorev.v1beta1.Query",
                "GetProtoRevAdminAccount",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_proto_rev_developer_account(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryGetProtoRevDeveloperAccountRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryGetProtoRevDeveloperAccountResponse>,
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
                "/osmosis.protorev.v1beta1.Query/GetProtoRevDeveloperAccount",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.protorev.v1beta1.Query",
                "GetProtoRevDeveloperAccount",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_proto_rev_info_by_pool_type(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryGetProtoRevInfoByPoolTypeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryGetProtoRevInfoByPoolTypeResponse>,
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
                "/osmosis.protorev.v1beta1.Query/GetProtoRevInfoByPoolType",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.protorev.v1beta1.Query",
                "GetProtoRevInfoByPoolType",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_proto_rev_max_pool_points_per_tx(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryGetProtoRevMaxPoolPointsPerTxRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryGetProtoRevMaxPoolPointsPerTxResponse>,
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
                "/osmosis.protorev.v1beta1.Query/GetProtoRevMaxPoolPointsPerTx",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.protorev.v1beta1.Query",
                "GetProtoRevMaxPoolPointsPerTx",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_proto_rev_max_pool_points_per_block(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryGetProtoRevMaxPoolPointsPerBlockRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryGetProtoRevMaxPoolPointsPerBlockResponse>,
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
                "/osmosis.protorev.v1beta1.Query/GetProtoRevMaxPoolPointsPerBlock",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.protorev.v1beta1.Query",
                "GetProtoRevMaxPoolPointsPerBlock",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_proto_rev_base_denoms(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryGetProtoRevBaseDenomsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryGetProtoRevBaseDenomsResponse>,
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
                "/osmosis.protorev.v1beta1.Query/GetProtoRevBaseDenoms",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.protorev.v1beta1.Query",
                "GetProtoRevBaseDenoms",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_proto_rev_enabled(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryGetProtoRevEnabledRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryGetProtoRevEnabledResponse>,
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
                "/osmosis.protorev.v1beta1.Query/GetProtoRevEnabled",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.protorev.v1beta1.Query",
                "GetProtoRevEnabled",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_proto_rev_pool(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryGetProtoRevPoolRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryGetProtoRevPoolResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/osmosis.protorev.v1beta1.Query/GetProtoRevPool",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.protorev.v1beta1.Query",
                "GetProtoRevPool",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_all_protocol_revenue(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryGetAllProtocolRevenueRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryGetAllProtocolRevenueResponse>,
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
                "/osmosis.protorev.v1beta1.Query/GetAllProtocolRevenue",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.protorev.v1beta1.Query",
                "GetAllProtocolRevenue",
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
        pub async fn set_hot_routes(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgSetHotRoutes>,
        ) -> std::result::Result<tonic::Response<super::MsgSetHotRoutesResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/osmosis.protorev.v1beta1.Msg/SetHotRoutes");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.protorev.v1beta1.Msg",
                "SetHotRoutes",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn set_developer_account(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgSetDeveloperAccount>,
        ) -> std::result::Result<
            tonic::Response<super::MsgSetDeveloperAccountResponse>,
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
                "/osmosis.protorev.v1beta1.Msg/SetDeveloperAccount",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.protorev.v1beta1.Msg",
                "SetDeveloperAccount",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn set_max_pool_points_per_tx(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgSetMaxPoolPointsPerTx>,
        ) -> std::result::Result<
            tonic::Response<super::MsgSetMaxPoolPointsPerTxResponse>,
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
                "/osmosis.protorev.v1beta1.Msg/SetMaxPoolPointsPerTx",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.protorev.v1beta1.Msg",
                "SetMaxPoolPointsPerTx",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn set_max_pool_points_per_block(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgSetMaxPoolPointsPerBlock>,
        ) -> std::result::Result<
            tonic::Response<super::MsgSetMaxPoolPointsPerBlockResponse>,
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
                "/osmosis.protorev.v1beta1.Msg/SetMaxPoolPointsPerBlock",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.protorev.v1beta1.Msg",
                "SetMaxPoolPointsPerBlock",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn set_info_by_pool_type(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgSetInfoByPoolType>,
        ) -> std::result::Result<tonic::Response<super::MsgSetInfoByPoolTypeResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/osmosis.protorev.v1beta1.Msg/SetInfoByPoolType",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.protorev.v1beta1.Msg",
                "SetInfoByPoolType",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn set_base_denoms(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgSetBaseDenoms>,
        ) -> std::result::Result<tonic::Response<super::MsgSetBaseDenomsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/osmosis.protorev.v1beta1.Msg/SetBaseDenoms");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.protorev.v1beta1.Msg",
                "SetBaseDenoms",
            ));
            self.inner.unary(req, path, codec).await
        }
    }
}
