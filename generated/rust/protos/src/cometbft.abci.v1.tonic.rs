// @generated
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod abci_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::{http::Uri, *};
    #[derive(Debug, Clone)]
    pub struct AbciServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl AbciServiceClient<tonic::transport::Channel> {
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
    impl<T> AbciServiceClient<T>
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
        ) -> AbciServiceClient<InterceptedService<T, F>>
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
            AbciServiceClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn echo(
            &mut self,
            request: impl tonic::IntoRequest<super::EchoRequest>,
        ) -> std::result::Result<tonic::Response<super::EchoResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cometbft.abci.v1.ABCIService/Echo");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cometbft.abci.v1.ABCIService", "Echo"));
            self.inner.unary(req, path, codec).await
        }
        /// Flush flushes the write buffer.
        pub async fn flush(
            &mut self,
            request: impl tonic::IntoRequest<super::FlushRequest>,
        ) -> std::result::Result<tonic::Response<super::FlushResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cometbft.abci.v1.ABCIService/Flush");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cometbft.abci.v1.ABCIService", "Flush"));
            self.inner.unary(req, path, codec).await
        }
        /// Info returns information about the application state.
        pub async fn info(
            &mut self,
            request: impl tonic::IntoRequest<super::InfoRequest>,
        ) -> std::result::Result<tonic::Response<super::InfoResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cometbft.abci.v1.ABCIService/Info");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cometbft.abci.v1.ABCIService", "Info"));
            self.inner.unary(req, path, codec).await
        }
        /// CheckTx validates a transaction.
        pub async fn check_tx(
            &mut self,
            request: impl tonic::IntoRequest<super::CheckTxRequest>,
        ) -> std::result::Result<tonic::Response<super::CheckTxResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cometbft.abci.v1.ABCIService/CheckTx");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cometbft.abci.v1.ABCIService", "CheckTx"));
            self.inner.unary(req, path, codec).await
        }
        /// Query queries the application state.
        pub async fn query(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cometbft.abci.v1.ABCIService/Query");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cometbft.abci.v1.ABCIService", "Query"));
            self.inner.unary(req, path, codec).await
        }
        /// Commit commits a block of transactions.
        pub async fn commit(
            &mut self,
            request: impl tonic::IntoRequest<super::CommitRequest>,
        ) -> std::result::Result<tonic::Response<super::CommitResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/cometbft.abci.v1.ABCIService/Commit");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cometbft.abci.v1.ABCIService", "Commit"));
            self.inner.unary(req, path, codec).await
        }
        /// InitChain initializes the blockchain.
        pub async fn init_chain(
            &mut self,
            request: impl tonic::IntoRequest<super::InitChainRequest>,
        ) -> std::result::Result<tonic::Response<super::InitChainResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cometbft.abci.v1.ABCIService/InitChain");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cometbft.abci.v1.ABCIService", "InitChain"));
            self.inner.unary(req, path, codec).await
        }
        /// ListSnapshots lists all the available snapshots.
        pub async fn list_snapshots(
            &mut self,
            request: impl tonic::IntoRequest<super::ListSnapshotsRequest>,
        ) -> std::result::Result<tonic::Response<super::ListSnapshotsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cometbft.abci.v1.ABCIService/ListSnapshots");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cometbft.abci.v1.ABCIService",
                "ListSnapshots",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// OfferSnapshot sends a snapshot offer.
        pub async fn offer_snapshot(
            &mut self,
            request: impl tonic::IntoRequest<super::OfferSnapshotRequest>,
        ) -> std::result::Result<tonic::Response<super::OfferSnapshotResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cometbft.abci.v1.ABCIService/OfferSnapshot");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cometbft.abci.v1.ABCIService",
                "OfferSnapshot",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// LoadSnapshotChunk returns a chunk of snapshot.
        pub async fn load_snapshot_chunk(
            &mut self,
            request: impl tonic::IntoRequest<super::LoadSnapshotChunkRequest>,
        ) -> std::result::Result<tonic::Response<super::LoadSnapshotChunkResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cometbft.abci.v1.ABCIService/LoadSnapshotChunk",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cometbft.abci.v1.ABCIService",
                "LoadSnapshotChunk",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// ApplySnapshotChunk applies a chunk of snapshot.
        pub async fn apply_snapshot_chunk(
            &mut self,
            request: impl tonic::IntoRequest<super::ApplySnapshotChunkRequest>,
        ) -> std::result::Result<tonic::Response<super::ApplySnapshotChunkResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cometbft.abci.v1.ABCIService/ApplySnapshotChunk",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cometbft.abci.v1.ABCIService",
                "ApplySnapshotChunk",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// PrepareProposal returns a proposal for the next block.
        pub async fn prepare_proposal(
            &mut self,
            request: impl tonic::IntoRequest<super::PrepareProposalRequest>,
        ) -> std::result::Result<tonic::Response<super::PrepareProposalResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cometbft.abci.v1.ABCIService/PrepareProposal",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cometbft.abci.v1.ABCIService",
                "PrepareProposal",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// ProcessProposal validates a proposal.
        pub async fn process_proposal(
            &mut self,
            request: impl tonic::IntoRequest<super::ProcessProposalRequest>,
        ) -> std::result::Result<tonic::Response<super::ProcessProposalResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cometbft.abci.v1.ABCIService/ProcessProposal",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cometbft.abci.v1.ABCIService",
                "ProcessProposal",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// ExtendVote extends a vote with application-injected data (vote extensions).
        pub async fn extend_vote(
            &mut self,
            request: impl tonic::IntoRequest<super::ExtendVoteRequest>,
        ) -> std::result::Result<tonic::Response<super::ExtendVoteResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cometbft.abci.v1.ABCIService/ExtendVote");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cometbft.abci.v1.ABCIService",
                "ExtendVote",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// VerifyVoteExtension verifies a vote extension.
        pub async fn verify_vote_extension(
            &mut self,
            request: impl tonic::IntoRequest<super::VerifyVoteExtensionRequest>,
        ) -> std::result::Result<tonic::Response<super::VerifyVoteExtensionResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cometbft.abci.v1.ABCIService/VerifyVoteExtension",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cometbft.abci.v1.ABCIService",
                "VerifyVoteExtension",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// FinalizeBlock finalizes a block.
        pub async fn finalize_block(
            &mut self,
            request: impl tonic::IntoRequest<super::FinalizeBlockRequest>,
        ) -> std::result::Result<tonic::Response<super::FinalizeBlockResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cometbft.abci.v1.ABCIService/FinalizeBlock");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cometbft.abci.v1.ABCIService",
                "FinalizeBlock",
            ));
            self.inner.unary(req, path, codec).await
        }
    }
}
