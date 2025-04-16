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
            let path = http::uri::PathAndQuery::from_static("/osmosis.superfluid.Query/Params");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("osmosis.superfluid.Query", "Params"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn asset_type(
            &mut self,
            request: impl tonic::IntoRequest<super::AssetTypeRequest>,
        ) -> std::result::Result<tonic::Response<super::AssetTypeResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/osmosis.superfluid.Query/AssetType");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("osmosis.superfluid.Query", "AssetType"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn all_assets(
            &mut self,
            request: impl tonic::IntoRequest<super::AllAssetsRequest>,
        ) -> std::result::Result<tonic::Response<super::AllAssetsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/osmosis.superfluid.Query/AllAssets");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("osmosis.superfluid.Query", "AllAssets"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn asset_multiplier(
            &mut self,
            request: impl tonic::IntoRequest<super::AssetMultiplierRequest>,
        ) -> std::result::Result<tonic::Response<super::AssetMultiplierResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/osmosis.superfluid.Query/AssetMultiplier");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.superfluid.Query",
                "AssetMultiplier",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn all_intermediary_accounts(
            &mut self,
            request: impl tonic::IntoRequest<super::AllIntermediaryAccountsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::AllIntermediaryAccountsResponse>,
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
                "/osmosis.superfluid.Query/AllIntermediaryAccounts",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.superfluid.Query",
                "AllIntermediaryAccounts",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn connected_intermediary_account(
            &mut self,
            request: impl tonic::IntoRequest<super::ConnectedIntermediaryAccountRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ConnectedIntermediaryAccountResponse>,
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
                "/osmosis.superfluid.Query/ConnectedIntermediaryAccount",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.superfluid.Query",
                "ConnectedIntermediaryAccount",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn total_delegation_by_validator_for_denom(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryTotalDelegationByValidatorForDenomRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryTotalDelegationByValidatorForDenomResponse>,
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
                "/osmosis.superfluid.Query/TotalDelegationByValidatorForDenom",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.superfluid.Query",
                "TotalDelegationByValidatorForDenom",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn total_superfluid_delegations(
            &mut self,
            request: impl tonic::IntoRequest<super::TotalSuperfluidDelegationsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TotalSuperfluidDelegationsResponse>,
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
                "/osmosis.superfluid.Query/TotalSuperfluidDelegations",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.superfluid.Query",
                "TotalSuperfluidDelegations",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn superfluid_delegation_amount(
            &mut self,
            request: impl tonic::IntoRequest<super::SuperfluidDelegationAmountRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SuperfluidDelegationAmountResponse>,
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
                "/osmosis.superfluid.Query/SuperfluidDelegationAmount",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.superfluid.Query",
                "SuperfluidDelegationAmount",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn superfluid_delegations_by_delegator(
            &mut self,
            request: impl tonic::IntoRequest<super::SuperfluidDelegationsByDelegatorRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SuperfluidDelegationsByDelegatorResponse>,
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
                "/osmosis.superfluid.Query/SuperfluidDelegationsByDelegator",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.superfluid.Query",
                "SuperfluidDelegationsByDelegator",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn superfluid_undelegations_by_delegator(
            &mut self,
            request: impl tonic::IntoRequest<super::SuperfluidUndelegationsByDelegatorRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SuperfluidUndelegationsByDelegatorResponse>,
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
                "/osmosis.superfluid.Query/SuperfluidUndelegationsByDelegator",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.superfluid.Query",
                "SuperfluidUndelegationsByDelegator",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn superfluid_delegations_by_validator_denom(
            &mut self,
            request: impl tonic::IntoRequest<super::SuperfluidDelegationsByValidatorDenomRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SuperfluidDelegationsByValidatorDenomResponse>,
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
                "/osmosis.superfluid.Query/SuperfluidDelegationsByValidatorDenom",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.superfluid.Query",
                "SuperfluidDelegationsByValidatorDenom",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn estimate_superfluid_delegated_amount_by_validator_denom(
            &mut self,
            request: impl tonic::IntoRequest<
                super::EstimateSuperfluidDelegatedAmountByValidatorDenomRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<super::EstimateSuperfluidDelegatedAmountByValidatorDenomResponse>,
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
                "/osmosis.superfluid.Query/EstimateSuperfluidDelegatedAmountByValidatorDenom",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.superfluid.Query",
                "EstimateSuperfluidDelegatedAmountByValidatorDenom",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn total_delegation_by_delegator(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryTotalDelegationByDelegatorRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryTotalDelegationByDelegatorResponse>,
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
                "/osmosis.superfluid.Query/TotalDelegationByDelegator",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.superfluid.Query",
                "TotalDelegationByDelegator",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn unpool_whitelist(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryUnpoolWhitelistRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryUnpoolWhitelistResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/osmosis.superfluid.Query/UnpoolWhitelist");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.superfluid.Query",
                "UnpoolWhitelist",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn user_concentrated_superfluid_positions_delegated(
            &mut self,
            request: impl tonic::IntoRequest<super::UserConcentratedSuperfluidPositionsDelegatedRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserConcentratedSuperfluidPositionsDelegatedResponse>,
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
                "/osmosis.superfluid.Query/UserConcentratedSuperfluidPositionsDelegated",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.superfluid.Query",
                "UserConcentratedSuperfluidPositionsDelegated",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn user_concentrated_superfluid_positions_undelegating(
            &mut self,
            request: impl tonic::IntoRequest<
                super::UserConcentratedSuperfluidPositionsUndelegatingRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<super::UserConcentratedSuperfluidPositionsUndelegatingResponse>,
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
                "/osmosis.superfluid.Query/UserConcentratedSuperfluidPositionsUndelegating",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.superfluid.Query",
                "UserConcentratedSuperfluidPositionsUndelegating",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn rest_supply(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryRestSupplyRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryRestSupplyResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/osmosis.superfluid.Query/RestSupply");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("osmosis.superfluid.Query", "RestSupply"));
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
        pub async fn superfluid_delegate(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgSuperfluidDelegate>,
        ) -> std::result::Result<tonic::Response<super::MsgSuperfluidDelegateResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/osmosis.superfluid.Msg/SuperfluidDelegate");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.superfluid.Msg",
                "SuperfluidDelegate",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn superfluid_undelegate(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgSuperfluidUndelegate>,
        ) -> std::result::Result<
            tonic::Response<super::MsgSuperfluidUndelegateResponse>,
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
                "/osmosis.superfluid.Msg/SuperfluidUndelegate",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.superfluid.Msg",
                "SuperfluidUndelegate",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn superfluid_unbond_lock(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgSuperfluidUnbondLock>,
        ) -> std::result::Result<
            tonic::Response<super::MsgSuperfluidUnbondLockResponse>,
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
                "/osmosis.superfluid.Msg/SuperfluidUnbondLock",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.superfluid.Msg",
                "SuperfluidUnbondLock",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn superfluid_undelegate_and_unbond_lock(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgSuperfluidUndelegateAndUnbondLock>,
        ) -> std::result::Result<
            tonic::Response<super::MsgSuperfluidUndelegateAndUnbondLockResponse>,
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
                "/osmosis.superfluid.Msg/SuperfluidUndelegateAndUnbondLock",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.superfluid.Msg",
                "SuperfluidUndelegateAndUnbondLock",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn lock_and_superfluid_delegate(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgLockAndSuperfluidDelegate>,
        ) -> std::result::Result<
            tonic::Response<super::MsgLockAndSuperfluidDelegateResponse>,
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
                "/osmosis.superfluid.Msg/LockAndSuperfluidDelegate",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.superfluid.Msg",
                "LockAndSuperfluidDelegate",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn create_full_range_position_and_superfluid_delegate(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgCreateFullRangePositionAndSuperfluidDelegate>,
        ) -> std::result::Result<
            tonic::Response<super::MsgCreateFullRangePositionAndSuperfluidDelegateResponse>,
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
                "/osmosis.superfluid.Msg/CreateFullRangePositionAndSuperfluidDelegate",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.superfluid.Msg",
                "CreateFullRangePositionAndSuperfluidDelegate",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn un_pool_whitelisted_pool(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgUnPoolWhitelistedPool>,
        ) -> std::result::Result<
            tonic::Response<super::MsgUnPoolWhitelistedPoolResponse>,
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
                "/osmosis.superfluid.Msg/UnPoolWhitelistedPool",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.superfluid.Msg",
                "UnPoolWhitelistedPool",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn unlock_and_migrate_shares_to_full_range_concentrated_position(
            &mut self,
            request: impl tonic::IntoRequest<
                super::MsgUnlockAndMigrateSharesToFullRangeConcentratedPosition,
            >,
        ) -> std::result::Result<
            tonic::Response<
                super::MsgUnlockAndMigrateSharesToFullRangeConcentratedPositionResponse,
            >,
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
                "/osmosis.superfluid.Msg/UnlockAndMigrateSharesToFullRangeConcentratedPosition",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.superfluid.Msg",
                "UnlockAndMigrateSharesToFullRangeConcentratedPosition",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn add_to_concentrated_liquidity_superfluid_position(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgAddToConcentratedLiquiditySuperfluidPosition>,
        ) -> std::result::Result<
            tonic::Response<super::MsgAddToConcentratedLiquiditySuperfluidPositionResponse>,
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
                "/osmosis.superfluid.Msg/AddToConcentratedLiquiditySuperfluidPosition",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.superfluid.Msg",
                "AddToConcentratedLiquiditySuperfluidPosition",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn unbond_convert_and_stake(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgUnbondConvertAndStake>,
        ) -> std::result::Result<
            tonic::Response<super::MsgUnbondConvertAndStakeResponse>,
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
                "/osmosis.superfluid.Msg/UnbondConvertAndStake",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "osmosis.superfluid.Msg",
                "UnbondConvertAndStake",
            ));
            self.inner.unary(req, path, codec).await
        }
    }
}
