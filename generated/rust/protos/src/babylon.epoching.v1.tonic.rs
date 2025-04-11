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
            let path = http::uri::PathAndQuery::from_static("/babylon.epoching.v1.Query/Params");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("babylon.epoching.v1.Query", "Params"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn epoch_info(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryEpochInfoRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryEpochInfoResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/babylon.epoching.v1.Query/EpochInfo");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("babylon.epoching.v1.Query", "EpochInfo"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn epochs_info(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryEpochsInfoRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryEpochsInfoResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/babylon.epoching.v1.Query/EpochsInfo");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("babylon.epoching.v1.Query", "EpochsInfo"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn current_epoch(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryCurrentEpochRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryCurrentEpochResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/babylon.epoching.v1.Query/CurrentEpoch");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("babylon.epoching.v1.Query", "CurrentEpoch"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn epoch_msgs(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryEpochMsgsRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryEpochMsgsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/babylon.epoching.v1.Query/EpochMsgs");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("babylon.epoching.v1.Query", "EpochMsgs"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn latest_epoch_msgs(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryLatestEpochMsgsRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryLatestEpochMsgsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/babylon.epoching.v1.Query/LatestEpochMsgs");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.epoching.v1.Query",
                "LatestEpochMsgs",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn validator_lifecycle(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryValidatorLifecycleRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryValidatorLifecycleResponse>,
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
                "/babylon.epoching.v1.Query/ValidatorLifecycle",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.epoching.v1.Query",
                "ValidatorLifecycle",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delegation_lifecycle(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryDelegationLifecycleRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryDelegationLifecycleResponse>,
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
                "/babylon.epoching.v1.Query/DelegationLifecycle",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.epoching.v1.Query",
                "DelegationLifecycle",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn epoch_val_set(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryEpochValSetRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryEpochValSetResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/babylon.epoching.v1.Query/EpochValSet");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("babylon.epoching.v1.Query", "EpochValSet"));
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
        pub async fn wrapped_delegate(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgWrappedDelegate>,
        ) -> std::result::Result<tonic::Response<super::MsgWrappedDelegateResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/babylon.epoching.v1.Msg/WrappedDelegate");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.epoching.v1.Msg",
                "WrappedDelegate",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn wrapped_undelegate(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgWrappedUndelegate>,
        ) -> std::result::Result<tonic::Response<super::MsgWrappedUndelegateResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/babylon.epoching.v1.Msg/WrappedUndelegate");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.epoching.v1.Msg",
                "WrappedUndelegate",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn wrapped_begin_redelegate(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgWrappedBeginRedelegate>,
        ) -> std::result::Result<
            tonic::Response<super::MsgWrappedBeginRedelegateResponse>,
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
                "/babylon.epoching.v1.Msg/WrappedBeginRedelegate",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.epoching.v1.Msg",
                "WrappedBeginRedelegate",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn wrapped_cancel_unbonding_delegation(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgWrappedCancelUnbondingDelegation>,
        ) -> std::result::Result<
            tonic::Response<super::MsgWrappedCancelUnbondingDelegationResponse>,
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
                "/babylon.epoching.v1.Msg/WrappedCancelUnbondingDelegation",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.epoching.v1.Msg",
                "WrappedCancelUnbondingDelegation",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn wrapped_edit_validator(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgWrappedEditValidator>,
        ) -> std::result::Result<
            tonic::Response<super::MsgWrappedEditValidatorResponse>,
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
                "/babylon.epoching.v1.Msg/WrappedEditValidator",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.epoching.v1.Msg",
                "WrappedEditValidator",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn wrapped_staking_update_params(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgWrappedStakingUpdateParams>,
        ) -> std::result::Result<
            tonic::Response<super::MsgWrappedStakingUpdateParamsResponse>,
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
                "/babylon.epoching.v1.Msg/WrappedStakingUpdateParams",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.epoching.v1.Msg",
                "WrappedStakingUpdateParams",
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
                http::uri::PathAndQuery::from_static("/babylon.epoching.v1.Msg/UpdateParams");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("babylon.epoching.v1.Msg", "UpdateParams"));
            self.inner.unary(req, path, codec).await
        }
    }
}
