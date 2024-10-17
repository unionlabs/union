// @generated
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod abci_application_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::{http::Uri, *};
    /// ABCIApplication is a service for an ABCI application.
    #[derive(Debug, Clone)]
    pub struct AbciApplicationClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl AbciApplicationClient<tonic::transport::Channel> {
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
    impl<T> AbciApplicationClient<T>
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
        ) -> AbciApplicationClient<InterceptedService<T, F>>
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
            AbciApplicationClient::new(InterceptedService::new(inner, interceptor))
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
        /// Echo returns back the same message it is sent.
        pub async fn echo(
            &mut self,
            request: impl tonic::IntoRequest<super::super::v1beta1::RequestEcho>,
        ) -> std::result::Result<tonic::Response<super::super::v1beta1::ResponseEcho>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cometbft.abci.v1beta2.ABCIApplication/Echo");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cometbft.abci.v1beta2.ABCIApplication",
                "Echo",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// Flush flushes the write buffer.
        pub async fn flush(
            &mut self,
            request: impl tonic::IntoRequest<super::super::v1beta1::RequestFlush>,
        ) -> std::result::Result<tonic::Response<super::super::v1beta1::ResponseFlush>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cometbft.abci.v1beta2.ABCIApplication/Flush",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cometbft.abci.v1beta2.ABCIApplication",
                "Flush",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// Info returns information about the application state.
        pub async fn info(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestInfo>,
        ) -> std::result::Result<tonic::Response<super::super::v1beta1::ResponseInfo>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/cometbft.abci.v1beta2.ABCIApplication/Info");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cometbft.abci.v1beta2.ABCIApplication",
                "Info",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// DeliverTx applies a transaction.
        pub async fn deliver_tx(
            &mut self,
            request: impl tonic::IntoRequest<super::super::v1beta1::RequestDeliverTx>,
        ) -> std::result::Result<tonic::Response<super::ResponseDeliverTx>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cometbft.abci.v1beta2.ABCIApplication/DeliverTx",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cometbft.abci.v1beta2.ABCIApplication",
                "DeliverTx",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// CheckTx validates a transaction.
        pub async fn check_tx(
            &mut self,
            request: impl tonic::IntoRequest<super::super::v1beta1::RequestCheckTx>,
        ) -> std::result::Result<tonic::Response<super::ResponseCheckTx>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cometbft.abci.v1beta2.ABCIApplication/CheckTx",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cometbft.abci.v1beta2.ABCIApplication",
                "CheckTx",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// Query queries the application state.
        pub async fn query(
            &mut self,
            request: impl tonic::IntoRequest<super::super::v1beta1::RequestQuery>,
        ) -> std::result::Result<tonic::Response<super::super::v1beta1::ResponseQuery>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cometbft.abci.v1beta2.ABCIApplication/Query",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cometbft.abci.v1beta2.ABCIApplication",
                "Query",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// Commit commits a block of transactions.
        pub async fn commit(
            &mut self,
            request: impl tonic::IntoRequest<super::super::v1beta1::RequestCommit>,
        ) -> std::result::Result<
            tonic::Response<super::super::v1beta1::ResponseCommit>,
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
                "/cometbft.abci.v1beta2.ABCIApplication/Commit",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cometbft.abci.v1beta2.ABCIApplication",
                "Commit",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// InitChain initializes the blockchain.
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
            let path = http::uri::PathAndQuery::from_static(
                "/cometbft.abci.v1beta2.ABCIApplication/InitChain",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cometbft.abci.v1beta2.ABCIApplication",
                "InitChain",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// BeginBlock signals the beginning of a block.
        pub async fn begin_block(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestBeginBlock>,
        ) -> std::result::Result<tonic::Response<super::ResponseBeginBlock>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cometbft.abci.v1beta2.ABCIApplication/BeginBlock",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cometbft.abci.v1beta2.ABCIApplication",
                "BeginBlock",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// EndBlock signals the end of a block, returns changes to the validator set.
        pub async fn end_block(
            &mut self,
            request: impl tonic::IntoRequest<super::super::v1beta1::RequestEndBlock>,
        ) -> std::result::Result<tonic::Response<super::ResponseEndBlock>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cometbft.abci.v1beta2.ABCIApplication/EndBlock",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cometbft.abci.v1beta2.ABCIApplication",
                "EndBlock",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// ListSnapshots lists all the available snapshots.
        pub async fn list_snapshots(
            &mut self,
            request: impl tonic::IntoRequest<super::super::v1beta1::RequestListSnapshots>,
        ) -> std::result::Result<
            tonic::Response<super::super::v1beta1::ResponseListSnapshots>,
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
                "/cometbft.abci.v1beta2.ABCIApplication/ListSnapshots",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cometbft.abci.v1beta2.ABCIApplication",
                "ListSnapshots",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// OfferSnapshot sends a snapshot offer.
        pub async fn offer_snapshot(
            &mut self,
            request: impl tonic::IntoRequest<super::super::v1beta1::RequestOfferSnapshot>,
        ) -> std::result::Result<
            tonic::Response<super::super::v1beta1::ResponseOfferSnapshot>,
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
                "/cometbft.abci.v1beta2.ABCIApplication/OfferSnapshot",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cometbft.abci.v1beta2.ABCIApplication",
                "OfferSnapshot",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// LoadSnapshotChunk returns a chunk of snapshot.
        pub async fn load_snapshot_chunk(
            &mut self,
            request: impl tonic::IntoRequest<super::super::v1beta1::RequestLoadSnapshotChunk>,
        ) -> std::result::Result<
            tonic::Response<super::super::v1beta1::ResponseLoadSnapshotChunk>,
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
                "/cometbft.abci.v1beta2.ABCIApplication/LoadSnapshotChunk",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cometbft.abci.v1beta2.ABCIApplication",
                "LoadSnapshotChunk",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// ApplySnapshotChunk applies a chunk of snapshot.
        pub async fn apply_snapshot_chunk(
            &mut self,
            request: impl tonic::IntoRequest<super::super::v1beta1::RequestApplySnapshotChunk>,
        ) -> std::result::Result<
            tonic::Response<super::super::v1beta1::ResponseApplySnapshotChunk>,
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
                "/cometbft.abci.v1beta2.ABCIApplication/ApplySnapshotChunk",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cometbft.abci.v1beta2.ABCIApplication",
                "ApplySnapshotChunk",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// PrepareProposal returns a proposal for the next block.
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
            let path = http::uri::PathAndQuery::from_static(
                "/cometbft.abci.v1beta2.ABCIApplication/PrepareProposal",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cometbft.abci.v1beta2.ABCIApplication",
                "PrepareProposal",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// ProcessProposal validates a proposal.
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
            let path = http::uri::PathAndQuery::from_static(
                "/cometbft.abci.v1beta2.ABCIApplication/ProcessProposal",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "cometbft.abci.v1beta2.ABCIApplication",
                "ProcessProposal",
            ));
            self.inner.unary(req, path, codec).await
        }
    }
}
