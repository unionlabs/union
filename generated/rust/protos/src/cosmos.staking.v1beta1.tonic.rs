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
        pub async fn create_validator(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgCreateValidator>,
        ) -> std::result::Result<tonic::Response<super::MsgCreateValidatorResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmos.staking.v1beta1.Msg/CreateValidator");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmos.staking.v1beta1.Msg",
                "CreateValidator",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn edit_validator(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgEditValidator>,
        ) -> std::result::Result<tonic::Response<super::MsgEditValidatorResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmos.staking.v1beta1.Msg/EditValidator");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmos.staking.v1beta1.Msg",
                "EditValidator",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delegate(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgDelegate>,
        ) -> std::result::Result<tonic::Response<super::MsgDelegateResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cosmos.staking.v1beta1.Msg/Delegate");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.staking.v1beta1.Msg", "Delegate"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn begin_redelegate(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgBeginRedelegate>,
        ) -> std::result::Result<tonic::Response<super::MsgBeginRedelegateResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmos.staking.v1beta1.Msg/BeginRedelegate");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmos.staking.v1beta1.Msg",
                "BeginRedelegate",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn undelegate(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgUndelegate>,
        ) -> std::result::Result<tonic::Response<super::MsgUndelegateResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmos.staking.v1beta1.Msg/Undelegate");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.staking.v1beta1.Msg", "Undelegate"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn cancel_unbonding_delegation(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgCancelUnbondingDelegation>,
        ) -> std::result::Result<
            tonic::Response<super::MsgCancelUnbondingDelegationResponse>,
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
                "/cosmos.staking.v1beta1.Msg/CancelUnbondingDelegation",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmos.staking.v1beta1.Msg",
                "CancelUnbondingDelegation",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_params(
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
            let path =
                http::uri::PathAndQuery::from_static("/cosmos.staking.v1beta1.Msg/UpdateParams");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmos.staking.v1beta1.Msg",
                "UpdateParams",
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
        pub async fn validators(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryValidatorsRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryValidatorsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmos.staking.v1beta1.Query/Validators");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmos.staking.v1beta1.Query",
                "Validators",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn validator(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryValidatorRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryValidatorResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmos.staking.v1beta1.Query/Validator");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.staking.v1beta1.Query", "Validator"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn validator_delegations(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryValidatorDelegationsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryValidatorDelegationsResponse>,
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
                "/cosmos.staking.v1beta1.Query/ValidatorDelegations",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmos.staking.v1beta1.Query",
                "ValidatorDelegations",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn validator_unbonding_delegations(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryValidatorUnbondingDelegationsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryValidatorUnbondingDelegationsResponse>,
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
                "/cosmos.staking.v1beta1.Query/ValidatorUnbondingDelegations",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmos.staking.v1beta1.Query",
                "ValidatorUnbondingDelegations",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delegation(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryDelegationRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryDelegationResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmos.staking.v1beta1.Query/Delegation");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmos.staking.v1beta1.Query",
                "Delegation",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn unbonding_delegation(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryUnbondingDelegationRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryUnbondingDelegationResponse>,
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
                "/cosmos.staking.v1beta1.Query/UnbondingDelegation",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmos.staking.v1beta1.Query",
                "UnbondingDelegation",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delegator_delegations(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryDelegatorDelegationsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryDelegatorDelegationsResponse>,
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
                "/cosmos.staking.v1beta1.Query/DelegatorDelegations",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmos.staking.v1beta1.Query",
                "DelegatorDelegations",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delegator_unbonding_delegations(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryDelegatorUnbondingDelegationsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryDelegatorUnbondingDelegationsResponse>,
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
                "/cosmos.staking.v1beta1.Query/DelegatorUnbondingDelegations",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmos.staking.v1beta1.Query",
                "DelegatorUnbondingDelegations",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn redelegations(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryRedelegationsRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryRedelegationsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cosmos.staking.v1beta1.Query/Redelegations");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmos.staking.v1beta1.Query",
                "Redelegations",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delegator_validators(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryDelegatorValidatorsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryDelegatorValidatorsResponse>,
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
                "/cosmos.staking.v1beta1.Query/DelegatorValidators",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmos.staking.v1beta1.Query",
                "DelegatorValidators",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delegator_validator(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryDelegatorValidatorRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryDelegatorValidatorResponse>,
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
                "/cosmos.staking.v1beta1.Query/DelegatorValidator",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmos.staking.v1beta1.Query",
                "DelegatorValidator",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn historical_info(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryHistoricalInfoRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryHistoricalInfoResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.staking.v1beta1.Query/HistoricalInfo",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmos.staking.v1beta1.Query",
                "HistoricalInfo",
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
            let path = http::uri::PathAndQuery::from_static("/cosmos.staking.v1beta1.Query/Pool");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.staking.v1beta1.Query", "Pool"));
            self.inner.unary(req, path, codec).await
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
            let path = http::uri::PathAndQuery::from_static("/cosmos.staking.v1beta1.Query/Params");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.staking.v1beta1.Query", "Params"));
            self.inner.unary(req, path, codec).await
        }
    }
}
