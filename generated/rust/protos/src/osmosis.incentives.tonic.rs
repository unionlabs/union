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
        pub async fn module_to_distribute_coins(
            &mut self,
            request: impl tonic::IntoRequest<super::ModuleToDistributeCoinsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ModuleToDistributeCoinsResponse>,
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
                "/osmosis.incentives.Query/ModuleToDistributeCoins",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.incentives.Query",
                "ModuleToDistributeCoins",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn gauge_by_id(
            &mut self,
            request: impl tonic::IntoRequest<super::GaugeByIdRequest>,
        ) -> std::result::Result<tonic::Response<super::GaugeByIdResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/osmosis.incentives.Query/GaugeByID");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("osmosis.incentives.Query", "GaugeByID"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn gauges(
            &mut self,
            request: impl tonic::IntoRequest<super::GaugesRequest>,
        ) -> std::result::Result<tonic::Response<super::GaugesResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/osmosis.incentives.Query/Gauges");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("osmosis.incentives.Query", "Gauges"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn active_gauges(
            &mut self,
            request: impl tonic::IntoRequest<super::ActiveGaugesRequest>,
        ) -> std::result::Result<tonic::Response<super::ActiveGaugesResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/osmosis.incentives.Query/ActiveGauges");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("osmosis.incentives.Query", "ActiveGauges"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn active_gauges_per_denom(
            &mut self,
            request: impl tonic::IntoRequest<super::ActiveGaugesPerDenomRequest>,
        ) -> std::result::Result<tonic::Response<super::ActiveGaugesPerDenomResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/osmosis.incentives.Query/ActiveGaugesPerDenom",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.incentives.Query",
                "ActiveGaugesPerDenom",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn upcoming_gauges(
            &mut self,
            request: impl tonic::IntoRequest<super::UpcomingGaugesRequest>,
        ) -> std::result::Result<tonic::Response<super::UpcomingGaugesResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/osmosis.incentives.Query/UpcomingGauges");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.incentives.Query",
                "UpcomingGauges",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn upcoming_gauges_per_denom(
            &mut self,
            request: impl tonic::IntoRequest<super::UpcomingGaugesPerDenomRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpcomingGaugesPerDenomResponse>,
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
                "/osmosis.incentives.Query/UpcomingGaugesPerDenom",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.incentives.Query",
                "UpcomingGaugesPerDenom",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn rewards_est(
            &mut self,
            request: impl tonic::IntoRequest<super::RewardsEstRequest>,
        ) -> std::result::Result<tonic::Response<super::RewardsEstResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/osmosis.incentives.Query/RewardsEst");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("osmosis.incentives.Query", "RewardsEst"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn lockable_durations(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryLockableDurationsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryLockableDurationsResponse>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/osmosis.incentives.Query/LockableDurations");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.incentives.Query",
                "LockableDurations",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn all_groups(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryAllGroupsRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryAllGroupsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/osmosis.incentives.Query/AllGroups");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("osmosis.incentives.Query", "AllGroups"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn all_groups_gauges(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryAllGroupsGaugesRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryAllGroupsGaugesResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/osmosis.incentives.Query/AllGroupsGauges");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.incentives.Query",
                "AllGroupsGauges",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn all_groups_with_gauge(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryAllGroupsWithGaugeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryAllGroupsWithGaugeResponse>,
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
                "/osmosis.incentives.Query/AllGroupsWithGauge",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.incentives.Query",
                "AllGroupsWithGauge",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn group_by_group_gauge_id(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryGroupByGroupGaugeIdRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryGroupByGroupGaugeIdResponse>,
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
                "/osmosis.incentives.Query/GroupByGroupGaugeID",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.incentives.Query",
                "GroupByGroupGaugeID",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn current_weight_by_group_gauge_id(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryCurrentWeightByGroupGaugeIdRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryCurrentWeightByGroupGaugeIdResponse>,
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
                "/osmosis.incentives.Query/CurrentWeightByGroupGaugeID",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.incentives.Query",
                "CurrentWeightByGroupGaugeID",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn internal_gauges(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryInternalGaugesRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryInternalGaugesResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/osmosis.incentives.Query/InternalGauges");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.incentives.Query",
                "InternalGauges",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn external_gauges(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryExternalGaugesRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryExternalGaugesResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/osmosis.incentives.Query/ExternalGauges");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.incentives.Query",
                "ExternalGauges",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn gauges_by_pool_id(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryGaugesByPoolIdRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryGaugesByPoolIdResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/osmosis.incentives.Query/GaugesByPoolID");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.incentives.Query",
                "GaugesByPoolID",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn params(
            &mut self,
            request: impl tonic::IntoRequest<super::ParamsRequest>,
        ) -> std::result::Result<tonic::Response<super::ParamsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/osmosis.incentives.Query/Params");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("osmosis.incentives.Query", "Params"));
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
        pub async fn create_gauge(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgCreateGauge>,
        ) -> std::result::Result<tonic::Response<super::MsgCreateGaugeResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/osmosis.incentives.Msg/CreateGauge");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("osmosis.incentives.Msg", "CreateGauge"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn add_to_gauge(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgAddToGauge>,
        ) -> std::result::Result<tonic::Response<super::MsgAddToGaugeResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/osmosis.incentives.Msg/AddToGauge");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("osmosis.incentives.Msg", "AddToGauge"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn create_group(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgCreateGroup>,
        ) -> std::result::Result<tonic::Response<super::MsgCreateGroupResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/osmosis.incentives.Msg/CreateGroup");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("osmosis.incentives.Msg", "CreateGroup"));
            self.inner.unary(req, path, codec).await
        }
    }
}
