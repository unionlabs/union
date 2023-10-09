// @generated
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod abci_application_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::{http::Uri, *};
    ///
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
        ///
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
            let path =
                http::uri::PathAndQuery::from_static("/tendermint.abci.ABCIApplication/Echo");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCIApplication", "Echo"));
            self.inner.unary(req, path, codec).await
        }
        ///
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
            let path =
                http::uri::PathAndQuery::from_static("/tendermint.abci.ABCIApplication/Flush");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCIApplication", "Flush"));
            self.inner.unary(req, path, codec).await
        }
        ///
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
            let path =
                http::uri::PathAndQuery::from_static("/tendermint.abci.ABCIApplication/Info");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCIApplication", "Info"));
            self.inner.unary(req, path, codec).await
        }
        ///
        pub async fn deliver_tx(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestDeliverTx>,
        ) -> std::result::Result<tonic::Response<super::ResponseDeliverTx>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/tendermint.abci.ABCIApplication/DeliverTx");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "tendermint.abci.ABCIApplication",
                "DeliverTx",
            ));
            self.inner.unary(req, path, codec).await
        }
        ///
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
            let path =
                http::uri::PathAndQuery::from_static("/tendermint.abci.ABCIApplication/CheckTx");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "tendermint.abci.ABCIApplication",
                "CheckTx",
            ));
            self.inner.unary(req, path, codec).await
        }
        ///
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
            let path =
                http::uri::PathAndQuery::from_static("/tendermint.abci.ABCIApplication/Query");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCIApplication", "Query"));
            self.inner.unary(req, path, codec).await
        }
        ///
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
            let path =
                http::uri::PathAndQuery::from_static("/tendermint.abci.ABCIApplication/Commit");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCIApplication", "Commit"));
            self.inner.unary(req, path, codec).await
        }
        ///
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
            let path =
                http::uri::PathAndQuery::from_static("/tendermint.abci.ABCIApplication/InitChain");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "tendermint.abci.ABCIApplication",
                "InitChain",
            ));
            self.inner.unary(req, path, codec).await
        }
        ///
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
            let path =
                http::uri::PathAndQuery::from_static("/tendermint.abci.ABCIApplication/BeginBlock");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "tendermint.abci.ABCIApplication",
                "BeginBlock",
            ));
            self.inner.unary(req, path, codec).await
        }
        ///
        pub async fn end_block(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestEndBlock>,
        ) -> std::result::Result<tonic::Response<super::ResponseEndBlock>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/tendermint.abci.ABCIApplication/EndBlock");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "tendermint.abci.ABCIApplication",
                "EndBlock",
            ));
            self.inner.unary(req, path, codec).await
        }
        ///
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
            let path = http::uri::PathAndQuery::from_static(
                "/tendermint.abci.ABCIApplication/ListSnapshots",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "tendermint.abci.ABCIApplication",
                "ListSnapshots",
            ));
            self.inner.unary(req, path, codec).await
        }
        ///
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
            let path = http::uri::PathAndQuery::from_static(
                "/tendermint.abci.ABCIApplication/OfferSnapshot",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "tendermint.abci.ABCIApplication",
                "OfferSnapshot",
            ));
            self.inner.unary(req, path, codec).await
        }
        ///
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
            let path = http::uri::PathAndQuery::from_static(
                "/tendermint.abci.ABCIApplication/LoadSnapshotChunk",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "tendermint.abci.ABCIApplication",
                "LoadSnapshotChunk",
            ));
            self.inner.unary(req, path, codec).await
        }
        ///
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
            let path = http::uri::PathAndQuery::from_static(
                "/tendermint.abci.ABCIApplication/ApplySnapshotChunk",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "tendermint.abci.ABCIApplication",
                "ApplySnapshotChunk",
            ));
            self.inner.unary(req, path, codec).await
        }
        ///
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
                "/tendermint.abci.ABCIApplication/PrepareProposal",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "tendermint.abci.ABCIApplication",
                "PrepareProposal",
            ));
            self.inner.unary(req, path, codec).await
        }
        ///
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
                "/tendermint.abci.ABCIApplication/ProcessProposal",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "tendermint.abci.ABCIApplication",
                "ProcessProposal",
            ));
            self.inner.unary(req, path, codec).await
        }
    }
}
