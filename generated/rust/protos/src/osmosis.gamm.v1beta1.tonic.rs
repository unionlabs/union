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
        pub async fn join_pool(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgJoinPool>,
        ) -> std::result::Result<tonic::Response<super::MsgJoinPoolResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/osmosis.gamm.v1beta1.Msg/JoinPool");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("osmosis.gamm.v1beta1.Msg", "JoinPool"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn exit_pool(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgExitPool>,
        ) -> std::result::Result<tonic::Response<super::MsgExitPoolResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/osmosis.gamm.v1beta1.Msg/ExitPool");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("osmosis.gamm.v1beta1.Msg", "ExitPool"));
            self.inner.unary(req, path, codec).await
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
            let path =
                http::uri::PathAndQuery::from_static("/osmosis.gamm.v1beta1.Msg/SwapExactAmountIn");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.gamm.v1beta1.Msg",
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
                "/osmosis.gamm.v1beta1.Msg/SwapExactAmountOut",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.gamm.v1beta1.Msg",
                "SwapExactAmountOut",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn join_swap_extern_amount_in(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgJoinSwapExternAmountIn>,
        ) -> std::result::Result<
            tonic::Response<super::MsgJoinSwapExternAmountInResponse>,
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
                "/osmosis.gamm.v1beta1.Msg/JoinSwapExternAmountIn",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.gamm.v1beta1.Msg",
                "JoinSwapExternAmountIn",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn join_swap_share_amount_out(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgJoinSwapShareAmountOut>,
        ) -> std::result::Result<
            tonic::Response<super::MsgJoinSwapShareAmountOutResponse>,
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
                "/osmosis.gamm.v1beta1.Msg/JoinSwapShareAmountOut",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.gamm.v1beta1.Msg",
                "JoinSwapShareAmountOut",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn exit_swap_extern_amount_out(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgExitSwapExternAmountOut>,
        ) -> std::result::Result<
            tonic::Response<super::MsgExitSwapExternAmountOutResponse>,
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
                "/osmosis.gamm.v1beta1.Msg/ExitSwapExternAmountOut",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.gamm.v1beta1.Msg",
                "ExitSwapExternAmountOut",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn exit_swap_share_amount_in(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgExitSwapShareAmountIn>,
        ) -> std::result::Result<
            tonic::Response<super::MsgExitSwapShareAmountInResponse>,
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
                "/osmosis.gamm.v1beta1.Msg/ExitSwapShareAmountIn",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.gamm.v1beta1.Msg",
                "ExitSwapShareAmountIn",
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
        pub async fn pools(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryPoolsRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryPoolsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/osmosis.gamm.v1beta1.Query/Pools");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("osmosis.gamm.v1beta1.Query", "Pools"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn num_pools(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryNumPoolsRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryNumPoolsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/osmosis.gamm.v1beta1.Query/NumPools");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("osmosis.gamm.v1beta1.Query", "NumPools"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn total_liquidity(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryTotalLiquidityRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryTotalLiquidityResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/osmosis.gamm.v1beta1.Query/TotalLiquidity");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.gamm.v1beta1.Query",
                "TotalLiquidity",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn pools_with_filter(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryPoolsWithFilterRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryPoolsWithFilterResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/osmosis.gamm.v1beta1.Query/PoolsWithFilter");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.gamm.v1beta1.Query",
                "PoolsWithFilter",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn pool(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryPoolRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryPoolResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/osmosis.gamm.v1beta1.Query/Pool");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("osmosis.gamm.v1beta1.Query", "Pool"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn pool_type(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryPoolTypeRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryPoolTypeResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/osmosis.gamm.v1beta1.Query/PoolType");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("osmosis.gamm.v1beta1.Query", "PoolType"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn calc_join_pool_no_swap_shares(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryCalcJoinPoolNoSwapSharesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryCalcJoinPoolNoSwapSharesResponse>,
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
                "/osmosis.gamm.v1beta1.Query/CalcJoinPoolNoSwapShares",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.gamm.v1beta1.Query",
                "CalcJoinPoolNoSwapShares",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn calc_join_pool_shares(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryCalcJoinPoolSharesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryCalcJoinPoolSharesResponse>,
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
                "/osmosis.gamm.v1beta1.Query/CalcJoinPoolShares",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.gamm.v1beta1.Query",
                "CalcJoinPoolShares",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn calc_exit_pool_coins_from_shares(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryCalcExitPoolCoinsFromSharesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryCalcExitPoolCoinsFromSharesResponse>,
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
                "/osmosis.gamm.v1beta1.Query/CalcExitPoolCoinsFromShares",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.gamm.v1beta1.Query",
                "CalcExitPoolCoinsFromShares",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn pool_params(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryPoolParamsRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryPoolParamsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/osmosis.gamm.v1beta1.Query/PoolParams");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("osmosis.gamm.v1beta1.Query", "PoolParams"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn total_pool_liquidity(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryTotalPoolLiquidityRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryTotalPoolLiquidityResponse>,
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
                "/osmosis.gamm.v1beta1.Query/TotalPoolLiquidity",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.gamm.v1beta1.Query",
                "TotalPoolLiquidity",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn total_shares(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryTotalSharesRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryTotalSharesResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/osmosis.gamm.v1beta1.Query/TotalShares");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("osmosis.gamm.v1beta1.Query", "TotalShares"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn spot_price(
            &mut self,
            request: impl tonic::IntoRequest<super::QuerySpotPriceRequest>,
        ) -> std::result::Result<tonic::Response<super::QuerySpotPriceResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/osmosis.gamm.v1beta1.Query/SpotPrice");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("osmosis.gamm.v1beta1.Query", "SpotPrice"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn estimate_swap_exact_amount_in(
            &mut self,
            request: impl tonic::IntoRequest<super::QuerySwapExactAmountInRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QuerySwapExactAmountInResponse>,
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
                "/osmosis.gamm.v1beta1.Query/EstimateSwapExactAmountIn",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.gamm.v1beta1.Query",
                "EstimateSwapExactAmountIn",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn estimate_swap_exact_amount_out(
            &mut self,
            request: impl tonic::IntoRequest<super::QuerySwapExactAmountOutRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QuerySwapExactAmountOutResponse>,
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
                "/osmosis.gamm.v1beta1.Query/EstimateSwapExactAmountOut",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.gamm.v1beta1.Query",
                "EstimateSwapExactAmountOut",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn concentrated_pool_id_link_from_cfmm(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryConcentratedPoolIdLinkFromCfmmRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryConcentratedPoolIdLinkFromCfmmResponse>,
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
                "/osmosis.gamm.v1beta1.Query/ConcentratedPoolIdLinkFromCFMM",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.gamm.v1beta1.Query",
                "ConcentratedPoolIdLinkFromCFMM",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn cfmm_concentrated_pool_links(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryCfmmConcentratedPoolLinksRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryCfmmConcentratedPoolLinksResponse>,
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
                "/osmosis.gamm.v1beta1.Query/CFMMConcentratedPoolLinks",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.gamm.v1beta1.Query",
                "CFMMConcentratedPoolLinks",
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
            let path = http::uri::PathAndQuery::from_static("/osmosis.gamm.v1beta1.Query/Params");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("osmosis.gamm.v1beta1.Query", "Params"));
            self.inner.unary(req, path, codec).await
        }
    }
}
