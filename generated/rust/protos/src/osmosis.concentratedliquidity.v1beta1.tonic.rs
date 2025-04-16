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
        pub async fn pools(
            &mut self,
            request: impl tonic::IntoRequest<super::PoolsRequest>,
        ) -> std::result::Result<tonic::Response<super::PoolsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/osmosis.concentratedliquidity.v1beta1.Query/Pools",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.concentratedliquidity.v1beta1.Query",
                "Pools",
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
            let path = http::uri::PathAndQuery::from_static(
                "/osmosis.concentratedliquidity.v1beta1.Query/Params",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.concentratedliquidity.v1beta1.Query",
                "Params",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn user_positions(
            &mut self,
            request: impl tonic::IntoRequest<super::UserPositionsRequest>,
        ) -> std::result::Result<tonic::Response<super::UserPositionsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/osmosis.concentratedliquidity.v1beta1.Query/UserPositions",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.concentratedliquidity.v1beta1.Query",
                "UserPositions",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn liquidity_per_tick_range(
            &mut self,
            request: impl tonic::IntoRequest<super::LiquidityPerTickRangeRequest>,
        ) -> std::result::Result<tonic::Response<super::LiquidityPerTickRangeResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/osmosis.concentratedliquidity.v1beta1.Query/LiquidityPerTickRange",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.concentratedliquidity.v1beta1.Query",
                "LiquidityPerTickRange",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn liquidity_net_in_direction(
            &mut self,
            request: impl tonic::IntoRequest<super::LiquidityNetInDirectionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::LiquidityNetInDirectionResponse>,
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
                "/osmosis.concentratedliquidity.v1beta1.Query/LiquidityNetInDirection",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.concentratedliquidity.v1beta1.Query",
                "LiquidityNetInDirection",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn claimable_spread_rewards(
            &mut self,
            request: impl tonic::IntoRequest<super::ClaimableSpreadRewardsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ClaimableSpreadRewardsResponse>,
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
                "/osmosis.concentratedliquidity.v1beta1.Query/ClaimableSpreadRewards",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.concentratedliquidity.v1beta1.Query",
                "ClaimableSpreadRewards",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn claimable_incentives(
            &mut self,
            request: impl tonic::IntoRequest<super::ClaimableIncentivesRequest>,
        ) -> std::result::Result<tonic::Response<super::ClaimableIncentivesResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/osmosis.concentratedliquidity.v1beta1.Query/ClaimableIncentives",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.concentratedliquidity.v1beta1.Query",
                "ClaimableIncentives",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn position_by_id(
            &mut self,
            request: impl tonic::IntoRequest<super::PositionByIdRequest>,
        ) -> std::result::Result<tonic::Response<super::PositionByIdResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/osmosis.concentratedliquidity.v1beta1.Query/PositionById",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.concentratedliquidity.v1beta1.Query",
                "PositionById",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn pool_accumulator_rewards(
            &mut self,
            request: impl tonic::IntoRequest<super::PoolAccumulatorRewardsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::PoolAccumulatorRewardsResponse>,
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
                "/osmosis.concentratedliquidity.v1beta1.Query/PoolAccumulatorRewards",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.concentratedliquidity.v1beta1.Query",
                "PoolAccumulatorRewards",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn incentive_records(
            &mut self,
            request: impl tonic::IntoRequest<super::IncentiveRecordsRequest>,
        ) -> std::result::Result<tonic::Response<super::IncentiveRecordsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/osmosis.concentratedliquidity.v1beta1.Query/IncentiveRecords",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.concentratedliquidity.v1beta1.Query",
                "IncentiveRecords",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn tick_accumulator_trackers(
            &mut self,
            request: impl tonic::IntoRequest<super::TickAccumulatorTrackersRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TickAccumulatorTrackersResponse>,
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
                "/osmosis.concentratedliquidity.v1beta1.Query/TickAccumulatorTrackers",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.concentratedliquidity.v1beta1.Query",
                "TickAccumulatorTrackers",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn cfmm_pool_id_link_from_concentrated_pool_id(
            &mut self,
            request: impl tonic::IntoRequest<super::CfmmPoolIdLinkFromConcentratedPoolIdRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CfmmPoolIdLinkFromConcentratedPoolIdResponse>,
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
                "/osmosis.concentratedliquidity.v1beta1.Query/CFMMPoolIdLinkFromConcentratedPoolId",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.concentratedliquidity.v1beta1.Query",
                "CFMMPoolIdLinkFromConcentratedPoolId",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn user_unbonding_positions(
            &mut self,
            request: impl tonic::IntoRequest<super::UserUnbondingPositionsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserUnbondingPositionsResponse>,
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
                "/osmosis.concentratedliquidity.v1beta1.Query/UserUnbondingPositions",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.concentratedliquidity.v1beta1.Query",
                "UserUnbondingPositions",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_total_liquidity(
            &mut self,
            request: impl tonic::IntoRequest<super::GetTotalLiquidityRequest>,
        ) -> std::result::Result<tonic::Response<super::GetTotalLiquidityResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/osmosis.concentratedliquidity.v1beta1.Query/GetTotalLiquidity",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.concentratedliquidity.v1beta1.Query",
                "GetTotalLiquidity",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn num_next_initialized_ticks(
            &mut self,
            request: impl tonic::IntoRequest<super::NumNextInitializedTicksRequest>,
        ) -> std::result::Result<
            tonic::Response<super::NumNextInitializedTicksResponse>,
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
                "/osmosis.concentratedliquidity.v1beta1.Query/NumNextInitializedTicks",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.concentratedliquidity.v1beta1.Query",
                "NumNextInitializedTicks",
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
        pub async fn create_position(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgCreatePosition>,
        ) -> std::result::Result<tonic::Response<super::MsgCreatePositionResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/osmosis.concentratedliquidity.v1beta1.Msg/CreatePosition",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.concentratedliquidity.v1beta1.Msg",
                "CreatePosition",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn withdraw_position(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgWithdrawPosition>,
        ) -> std::result::Result<tonic::Response<super::MsgWithdrawPositionResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/osmosis.concentratedliquidity.v1beta1.Msg/WithdrawPosition",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.concentratedliquidity.v1beta1.Msg",
                "WithdrawPosition",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn add_to_position(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgAddToPosition>,
        ) -> std::result::Result<tonic::Response<super::MsgAddToPositionResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/osmosis.concentratedliquidity.v1beta1.Msg/AddToPosition",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.concentratedliquidity.v1beta1.Msg",
                "AddToPosition",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn collect_spread_rewards(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgCollectSpreadRewards>,
        ) -> std::result::Result<
            tonic::Response<super::MsgCollectSpreadRewardsResponse>,
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
                "/osmosis.concentratedliquidity.v1beta1.Msg/CollectSpreadRewards",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.concentratedliquidity.v1beta1.Msg",
                "CollectSpreadRewards",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn collect_incentives(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgCollectIncentives>,
        ) -> std::result::Result<tonic::Response<super::MsgCollectIncentivesResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/osmosis.concentratedliquidity.v1beta1.Msg/CollectIncentives",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.concentratedliquidity.v1beta1.Msg",
                "CollectIncentives",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn transfer_positions(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgTransferPositions>,
        ) -> std::result::Result<tonic::Response<super::MsgTransferPositionsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/osmosis.concentratedliquidity.v1beta1.Msg/TransferPositions",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.concentratedliquidity.v1beta1.Msg",
                "TransferPositions",
            ));
            self.inner.unary(req, path, codec).await
        }
    }
}
