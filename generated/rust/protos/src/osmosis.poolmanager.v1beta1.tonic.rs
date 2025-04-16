// @generated
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
        pub async fn swap_exact_amount_in(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgSwapExactAmountIn>,
        ) -> std::result::Result<tonic::Response<super::MsgSwapExactAmountInResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/osmosis.poolmanager.v1beta1.Msg/SwapExactAmountIn",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Msg",
                "SwapExactAmountIn",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn swap_exact_amount_out(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgSwapExactAmountOut>,
        ) -> std::result::Result<tonic::Response<super::MsgSwapExactAmountOutResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/osmosis.poolmanager.v1beta1.Msg/SwapExactAmountOut",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Msg",
                "SwapExactAmountOut",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn split_route_swap_exact_amount_in(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgSplitRouteSwapExactAmountIn>,
        ) -> std::result::Result<
            tonic::Response<super::MsgSplitRouteSwapExactAmountInResponse>,
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
                "/osmosis.poolmanager.v1beta1.Msg/SplitRouteSwapExactAmountIn",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Msg",
                "SplitRouteSwapExactAmountIn",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn split_route_swap_exact_amount_out(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgSplitRouteSwapExactAmountOut>,
        ) -> std::result::Result<
            tonic::Response<super::MsgSplitRouteSwapExactAmountOutResponse>,
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
                "/osmosis.poolmanager.v1beta1.Msg/SplitRouteSwapExactAmountOut",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Msg",
                "SplitRouteSwapExactAmountOut",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn set_denom_pair_taker_fee(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgSetDenomPairTakerFee>,
        ) -> std::result::Result<
            tonic::Response<super::MsgSetDenomPairTakerFeeResponse>,
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
                "/osmosis.poolmanager.v1beta1.Msg/SetDenomPairTakerFee",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Msg",
                "SetDenomPairTakerFee",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn set_taker_fee_share_agreement_for_denom(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgSetTakerFeeShareAgreementForDenom>,
        ) -> std::result::Result<
            tonic::Response<super::MsgSetTakerFeeShareAgreementForDenomResponse>,
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
                "/osmosis.poolmanager.v1beta1.Msg/SetTakerFeeShareAgreementForDenom",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Msg",
                "SetTakerFeeShareAgreementForDenom",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn set_registered_alloyed_pool(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgSetRegisteredAlloyedPool>,
        ) -> std::result::Result<
            tonic::Response<super::MsgSetRegisteredAlloyedPoolResponse>,
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
                "/osmosis.poolmanager.v1beta1.Msg/SetRegisteredAlloyedPool",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Msg",
                "SetRegisteredAlloyedPool",
            ));
            self.inner.unary(req, path, codec).await
        }
    }
}
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
            request: impl tonic::IntoRequest<super::ParamsRequest>,
        ) -> std::result::Result<tonic::Response<super::ParamsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/osmosis.poolmanager.v1beta1.Query/Params");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Query",
                "Params",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn estimate_swap_exact_amount_in(
            &mut self,
            request: impl tonic::IntoRequest<super::EstimateSwapExactAmountInRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EstimateSwapExactAmountInResponse>,
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
                "/osmosis.poolmanager.v1beta1.Query/EstimateSwapExactAmountIn",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Query",
                "EstimateSwapExactAmountIn",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn estimate_swap_exact_amount_in_with_primitive_types(
            &mut self,
            request: impl tonic::IntoRequest<super::EstimateSwapExactAmountInWithPrimitiveTypesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EstimateSwapExactAmountInResponse>,
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
                "/osmosis.poolmanager.v1beta1.Query/EstimateSwapExactAmountInWithPrimitiveTypes",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Query",
                "EstimateSwapExactAmountInWithPrimitiveTypes",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn estimate_single_pool_swap_exact_amount_in(
            &mut self,
            request: impl tonic::IntoRequest<super::EstimateSinglePoolSwapExactAmountInRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EstimateSwapExactAmountInResponse>,
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
                "/osmosis.poolmanager.v1beta1.Query/EstimateSinglePoolSwapExactAmountIn",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Query",
                "EstimateSinglePoolSwapExactAmountIn",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn estimate_swap_exact_amount_out(
            &mut self,
            request: impl tonic::IntoRequest<super::EstimateSwapExactAmountOutRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EstimateSwapExactAmountOutResponse>,
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
                "/osmosis.poolmanager.v1beta1.Query/EstimateSwapExactAmountOut",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Query",
                "EstimateSwapExactAmountOut",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn estimate_swap_exact_amount_out_with_primitive_types(
            &mut self,
            request: impl tonic::IntoRequest<super::EstimateSwapExactAmountOutWithPrimitiveTypesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EstimateSwapExactAmountOutResponse>,
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
                "/osmosis.poolmanager.v1beta1.Query/EstimateSwapExactAmountOutWithPrimitiveTypes",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Query",
                "EstimateSwapExactAmountOutWithPrimitiveTypes",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn estimate_single_pool_swap_exact_amount_out(
            &mut self,
            request: impl tonic::IntoRequest<super::EstimateSinglePoolSwapExactAmountOutRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EstimateSwapExactAmountOutResponse>,
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
                "/osmosis.poolmanager.v1beta1.Query/EstimateSinglePoolSwapExactAmountOut",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Query",
                "EstimateSinglePoolSwapExactAmountOut",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn num_pools(
            &mut self,
            request: impl tonic::IntoRequest<super::NumPoolsRequest>,
        ) -> std::result::Result<tonic::Response<super::NumPoolsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/osmosis.poolmanager.v1beta1.Query/NumPools");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Query",
                "NumPools",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn pool(
            &mut self,
            request: impl tonic::IntoRequest<super::PoolRequest>,
        ) -> std::result::Result<tonic::Response<super::PoolResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/osmosis.poolmanager.v1beta1.Query/Pool");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("osmosis.poolmanager.v1beta1.Query", "Pool"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn all_pools(
            &mut self,
            request: impl tonic::IntoRequest<super::AllPoolsRequest>,
        ) -> std::result::Result<tonic::Response<super::AllPoolsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/osmosis.poolmanager.v1beta1.Query/AllPools");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Query",
                "AllPools",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_pools_by_denom(
            &mut self,
            request: impl tonic::IntoRequest<super::ListPoolsByDenomRequest>,
        ) -> std::result::Result<tonic::Response<super::ListPoolsByDenomResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/osmosis.poolmanager.v1beta1.Query/ListPoolsByDenom",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Query",
                "ListPoolsByDenom",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn spot_price(
            &mut self,
            request: impl tonic::IntoRequest<super::SpotPriceRequest>,
        ) -> std::result::Result<tonic::Response<super::SpotPriceResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/osmosis.poolmanager.v1beta1.Query/SpotPrice",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Query",
                "SpotPrice",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn total_pool_liquidity(
            &mut self,
            request: impl tonic::IntoRequest<super::TotalPoolLiquidityRequest>,
        ) -> std::result::Result<tonic::Response<super::TotalPoolLiquidityResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/osmosis.poolmanager.v1beta1.Query/TotalPoolLiquidity",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Query",
                "TotalPoolLiquidity",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn total_liquidity(
            &mut self,
            request: impl tonic::IntoRequest<super::TotalLiquidityRequest>,
        ) -> std::result::Result<tonic::Response<super::TotalLiquidityResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/osmosis.poolmanager.v1beta1.Query/TotalLiquidity",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Query",
                "TotalLiquidity",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn total_volume_for_pool(
            &mut self,
            request: impl tonic::IntoRequest<super::TotalVolumeForPoolRequest>,
        ) -> std::result::Result<tonic::Response<super::TotalVolumeForPoolResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/osmosis.poolmanager.v1beta1.Query/TotalVolumeForPool",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Query",
                "TotalVolumeForPool",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn trading_pair_taker_fee(
            &mut self,
            request: impl tonic::IntoRequest<super::TradingPairTakerFeeRequest>,
        ) -> std::result::Result<tonic::Response<super::TradingPairTakerFeeResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/osmosis.poolmanager.v1beta1.Query/TradingPairTakerFee",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Query",
                "TradingPairTakerFee",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn estimate_trade_based_on_price_impact(
            &mut self,
            request: impl tonic::IntoRequest<super::EstimateTradeBasedOnPriceImpactRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EstimateTradeBasedOnPriceImpactResponse>,
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
                "/osmosis.poolmanager.v1beta1.Query/EstimateTradeBasedOnPriceImpact",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Query",
                "EstimateTradeBasedOnPriceImpact",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn all_taker_fee_share_agreements(
            &mut self,
            request: impl tonic::IntoRequest<super::AllTakerFeeShareAgreementsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::AllTakerFeeShareAgreementsResponse>,
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
                "/osmosis.poolmanager.v1beta1.Query/AllTakerFeeShareAgreements",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Query",
                "AllTakerFeeShareAgreements",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn taker_fee_share_agreement_from_denom(
            &mut self,
            request: impl tonic::IntoRequest<super::TakerFeeShareAgreementFromDenomRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TakerFeeShareAgreementFromDenomResponse>,
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
                "/osmosis.poolmanager.v1beta1.Query/TakerFeeShareAgreementFromDenom",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Query",
                "TakerFeeShareAgreementFromDenom",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn taker_fee_share_denoms_to_accrued_value(
            &mut self,
            request: impl tonic::IntoRequest<super::TakerFeeShareDenomsToAccruedValueRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TakerFeeShareDenomsToAccruedValueResponse>,
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
                "/osmosis.poolmanager.v1beta1.Query/TakerFeeShareDenomsToAccruedValue",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Query",
                "TakerFeeShareDenomsToAccruedValue",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn all_taker_fee_share_accumulators(
            &mut self,
            request: impl tonic::IntoRequest<super::AllTakerFeeShareAccumulatorsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::AllTakerFeeShareAccumulatorsResponse>,
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
                "/osmosis.poolmanager.v1beta1.Query/AllTakerFeeShareAccumulators",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Query",
                "AllTakerFeeShareAccumulators",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn registered_alloyed_pool_from_denom(
            &mut self,
            request: impl tonic::IntoRequest<super::RegisteredAlloyedPoolFromDenomRequest>,
        ) -> std::result::Result<
            tonic::Response<super::RegisteredAlloyedPoolFromDenomResponse>,
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
                "/osmosis.poolmanager.v1beta1.Query/RegisteredAlloyedPoolFromDenom",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Query",
                "RegisteredAlloyedPoolFromDenom",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn registered_alloyed_pool_from_pool_id(
            &mut self,
            request: impl tonic::IntoRequest<super::RegisteredAlloyedPoolFromPoolIdRequest>,
        ) -> std::result::Result<
            tonic::Response<super::RegisteredAlloyedPoolFromPoolIdResponse>,
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
                "/osmosis.poolmanager.v1beta1.Query/RegisteredAlloyedPoolFromPoolId",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Query",
                "RegisteredAlloyedPoolFromPoolId",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn all_registered_alloyed_pools(
            &mut self,
            request: impl tonic::IntoRequest<super::AllRegisteredAlloyedPoolsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::AllRegisteredAlloyedPoolsResponse>,
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
                "/osmosis.poolmanager.v1beta1.Query/AllRegisteredAlloyedPools",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.poolmanager.v1beta1.Query",
                "AllRegisteredAlloyedPools",
            ));
            self.inner.unary(req, path, codec).await
        }
    }
}
