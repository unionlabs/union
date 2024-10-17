// @generated
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod pruning_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::{http::Uri, *};
    /// PruningService provides privileged access to specialized pruning
    /// functionality on the CometBFT node to help control node storage.
    #[derive(Debug, Clone)]
    pub struct PruningServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl PruningServiceClient<tonic::transport::Channel> {
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
    impl<T> PruningServiceClient<T>
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
        ) -> PruningServiceClient<InterceptedService<T, F>>
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
            PruningServiceClient::new(InterceptedService::new(inner, interceptor))
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
        /// SetBlockRetainHeightRequest indicates to the node that it can safely
        /// prune all block data up to the specified retain height.
        ///
        /// The lower of this retain height and that set by the application in its
        /// Commit response will be used by the node to determine which heights' data
        /// can be pruned.
        pub async fn set_block_retain_height(
            &mut self,
            request: impl tonic::IntoRequest<super::SetBlockRetainHeightRequest>,
        ) -> std::result::Result<tonic::Response<super::SetBlockRetainHeightResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/tendermint.services.pruning.v1.PruningService/SetBlockRetainHeight",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "tendermint.services.pruning.v1.PruningService",
                "SetBlockRetainHeight",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// GetBlockRetainHeight returns information about the retain height
        /// parameters used by the node to influence block retention/pruning.
        pub async fn get_block_retain_height(
            &mut self,
            request: impl tonic::IntoRequest<super::GetBlockRetainHeightRequest>,
        ) -> std::result::Result<tonic::Response<super::GetBlockRetainHeightResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/tendermint.services.pruning.v1.PruningService/GetBlockRetainHeight",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "tendermint.services.pruning.v1.PruningService",
                "GetBlockRetainHeight",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// SetBlockResultsRetainHeightRequest indicates to the node that it can
        /// safely prune all block results data up to the specified height.
        ///
        /// The node will always store the block results for the latest height to
        /// help facilitate crash recovery.
        pub async fn set_block_results_retain_height(
            &mut self,
            request: impl tonic::IntoRequest<super::SetBlockResultsRetainHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SetBlockResultsRetainHeightResponse>,
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
                "/tendermint.services.pruning.v1.PruningService/SetBlockResultsRetainHeight",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "tendermint.services.pruning.v1.PruningService",
                "SetBlockResultsRetainHeight",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// GetBlockResultsRetainHeight returns information about the retain height
        /// parameters used by the node to influence block results retention/pruning.
        pub async fn get_block_results_retain_height(
            &mut self,
            request: impl tonic::IntoRequest<super::GetBlockResultsRetainHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetBlockResultsRetainHeightResponse>,
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
                "/tendermint.services.pruning.v1.PruningService/GetBlockResultsRetainHeight",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "tendermint.services.pruning.v1.PruningService",
                "GetBlockResultsRetainHeight",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// SetTxIndexerRetainHeightRequest indicates to the node that it can safely
        /// prune all tx indices up to the specified retain height.
        pub async fn set_tx_indexer_retain_height(
            &mut self,
            request: impl tonic::IntoRequest<super::SetTxIndexerRetainHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SetTxIndexerRetainHeightResponse>,
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
                "/tendermint.services.pruning.v1.PruningService/SetTxIndexerRetainHeight",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "tendermint.services.pruning.v1.PruningService",
                "SetTxIndexerRetainHeight",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// GetTxIndexerRetainHeight returns information about the retain height
        /// parameters used by the node to influence TxIndexer pruning
        pub async fn get_tx_indexer_retain_height(
            &mut self,
            request: impl tonic::IntoRequest<super::GetTxIndexerRetainHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetTxIndexerRetainHeightResponse>,
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
                "/tendermint.services.pruning.v1.PruningService/GetTxIndexerRetainHeight",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "tendermint.services.pruning.v1.PruningService",
                "GetTxIndexerRetainHeight",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// SetBlockIndexerRetainHeightRequest indicates to the node that it can safely
        /// prune all block indices up to the specified retain height.
        pub async fn set_block_indexer_retain_height(
            &mut self,
            request: impl tonic::IntoRequest<super::SetBlockIndexerRetainHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SetBlockIndexerRetainHeightResponse>,
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
                "/tendermint.services.pruning.v1.PruningService/SetBlockIndexerRetainHeight",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "tendermint.services.pruning.v1.PruningService",
                "SetBlockIndexerRetainHeight",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// GetBlockIndexerRetainHeight returns information about the retain height
        /// parameters used by the node to influence BlockIndexer pruning
        pub async fn get_block_indexer_retain_height(
            &mut self,
            request: impl tonic::IntoRequest<super::GetBlockIndexerRetainHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetBlockIndexerRetainHeightResponse>,
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
                "/tendermint.services.pruning.v1.PruningService/GetBlockIndexerRetainHeight",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "tendermint.services.pruning.v1.PruningService",
                "GetBlockIndexerRetainHeight",
            ));
            self.inner.unary(req, path, codec).await
        }
    }
}
