// @generated
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::{http::Uri, *};
    #[derive(Debug, Clone)]
    pub struct ServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ServiceClient<tonic::transport::Channel> {
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
    impl<T> ServiceClient<T>
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
        ) -> ServiceClient<InterceptedService<T, F>>
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
            ServiceClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn get_node_info(
            &mut self,
            request: impl tonic::IntoRequest<super::GetNodeInfoRequest>,
        ) -> std::result::Result<tonic::Response<super::GetNodeInfoResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.base.tendermint.v1beta1.Service/GetNodeInfo",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmos.base.tendermint.v1beta1.Service",
                "GetNodeInfo",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_syncing(
            &mut self,
            request: impl tonic::IntoRequest<super::GetSyncingRequest>,
        ) -> std::result::Result<tonic::Response<super::GetSyncingResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.base.tendermint.v1beta1.Service/GetSyncing",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmos.base.tendermint.v1beta1.Service",
                "GetSyncing",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_latest_block(
            &mut self,
            request: impl tonic::IntoRequest<super::GetLatestBlockRequest>,
        ) -> std::result::Result<tonic::Response<super::GetLatestBlockResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.base.tendermint.v1beta1.Service/GetLatestBlock",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmos.base.tendermint.v1beta1.Service",
                "GetLatestBlock",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_block_by_height(
            &mut self,
            request: impl tonic::IntoRequest<super::GetBlockByHeightRequest>,
        ) -> std::result::Result<tonic::Response<super::GetBlockByHeightResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.base.tendermint.v1beta1.Service/GetBlockByHeight",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmos.base.tendermint.v1beta1.Service",
                "GetBlockByHeight",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_latest_validator_set(
            &mut self,
            request: impl tonic::IntoRequest<super::GetLatestValidatorSetRequest>,
        ) -> std::result::Result<tonic::Response<super::GetLatestValidatorSetResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.base.tendermint.v1beta1.Service/GetLatestValidatorSet",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmos.base.tendermint.v1beta1.Service",
                "GetLatestValidatorSet",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_validator_set_by_height(
            &mut self,
            request: impl tonic::IntoRequest<super::GetValidatorSetByHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetValidatorSetByHeightResponse>,
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
                "/cosmos.base.tendermint.v1beta1.Service/GetValidatorSetByHeight",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmos.base.tendermint.v1beta1.Service",
                "GetValidatorSetByHeight",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn abci_query(
            &mut self,
            request: impl tonic::IntoRequest<super::AbciQueryRequest>,
        ) -> std::result::Result<tonic::Response<super::AbciQueryResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.base.tendermint.v1beta1.Service/ABCIQuery",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cosmos.base.tendermint.v1beta1.Service",
                "ABCIQuery",
            ));
            self.inner.unary(req, path, codec).await
        }
    }
}
