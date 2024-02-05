// @generated
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod abci_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::{http::Uri, *};
    #[derive(Debug, Clone)]
    pub struct AbciClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl AbciClient<tonic::transport::Channel> {
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
    impl<T> AbciClient<T>
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
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> AbciClient<InterceptedService<T, F>>
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
            AbciClient::new(InterceptedService::new(inner, interceptor))
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
            request: impl tonic::IntoRequest<super::RequestEcho>,
        ) -> std::result::Result<tonic::Response<super::ResponseEcho>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/tendermint.abci.ABCI/Echo");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCI", "Echo"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn flush(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestFlush>,
        ) -> std::result::Result<tonic::Response<super::ResponseFlush>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/tendermint.abci.ABCI/Flush");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCI", "Flush"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn info(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestInfo>,
        ) -> std::result::Result<tonic::Response<super::ResponseInfo>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/tendermint.abci.ABCI/Info");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCI", "Info"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn check_tx(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestCheckTx>,
        ) -> std::result::Result<tonic::Response<super::ResponseCheckTx>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/tendermint.abci.ABCI/CheckTx");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCI", "CheckTx"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn query(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestQuery>,
        ) -> std::result::Result<tonic::Response<super::ResponseQuery>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/tendermint.abci.ABCI/Query");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCI", "Query"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn commit(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestCommit>,
        ) -> std::result::Result<tonic::Response<super::ResponseCommit>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/tendermint.abci.ABCI/Commit");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCI", "Commit"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn init_chain(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestInitChain>,
        ) -> std::result::Result<tonic::Response<super::ResponseInitChain>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/tendermint.abci.ABCI/InitChain");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCI", "InitChain"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_snapshots(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestListSnapshots>,
        ) -> std::result::Result<tonic::Response<super::ResponseListSnapshots>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/tendermint.abci.ABCI/ListSnapshots");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCI", "ListSnapshots"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn offer_snapshot(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestOfferSnapshot>,
        ) -> std::result::Result<tonic::Response<super::ResponseOfferSnapshot>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/tendermint.abci.ABCI/OfferSnapshot");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCI", "OfferSnapshot"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn load_snapshot_chunk(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestLoadSnapshotChunk>,
        ) -> std::result::Result<tonic::Response<super::ResponseLoadSnapshotChunk>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/tendermint.abci.ABCI/LoadSnapshotChunk");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCI", "LoadSnapshotChunk"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn apply_snapshot_chunk(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestApplySnapshotChunk>,
        ) -> std::result::Result<tonic::Response<super::ResponseApplySnapshotChunk>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/tendermint.abci.ABCI/ApplySnapshotChunk");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "tendermint.abci.ABCI",
                "ApplySnapshotChunk",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn prepare_proposal(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestPrepareProposal>,
        ) -> std::result::Result<tonic::Response<super::ResponsePrepareProposal>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/tendermint.abci.ABCI/PrepareProposal");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCI", "PrepareProposal"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn process_proposal(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestProcessProposal>,
        ) -> std::result::Result<tonic::Response<super::ResponseProcessProposal>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/tendermint.abci.ABCI/ProcessProposal");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCI", "ProcessProposal"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn extend_vote(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestExtendVote>,
        ) -> std::result::Result<tonic::Response<super::ResponseExtendVote>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/tendermint.abci.ABCI/ExtendVote");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCI", "ExtendVote"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn verify_vote_extension(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestVerifyVoteExtension>,
        ) -> std::result::Result<tonic::Response<super::ResponseVerifyVoteExtension>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/tendermint.abci.ABCI/VerifyVoteExtension");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "tendermint.abci.ABCI",
                "VerifyVoteExtension",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn finalize_block(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestFinalizeBlock>,
        ) -> std::result::Result<tonic::Response<super::ResponseFinalizeBlock>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/tendermint.abci.ABCI/FinalizeBlock");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCI", "FinalizeBlock"));
            self.inner.unary(req, path, codec).await
        }
    }
}
