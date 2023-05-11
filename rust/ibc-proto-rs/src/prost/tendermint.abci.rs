/// Generated client implementations.
#[cfg(feature = "client")]
pub mod abci_application_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
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
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
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
        pub async fn echo(
            &mut self,
            request: impl tonic::IntoRequest<::tendermint_proto::abci::RequestEcho>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseEcho>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/tendermint.abci.ABCIApplication/Echo",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCIApplication", "Echo"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn flush(
            &mut self,
            request: impl tonic::IntoRequest<::tendermint_proto::abci::RequestFlush>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseFlush>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/tendermint.abci.ABCIApplication/Flush",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCIApplication", "Flush"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn info(
            &mut self,
            request: impl tonic::IntoRequest<::tendermint_proto::abci::RequestInfo>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseInfo>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/tendermint.abci.ABCIApplication/Info",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCIApplication", "Info"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn deliver_tx(
            &mut self,
            request: impl tonic::IntoRequest<::tendermint_proto::abci::RequestDeliverTx>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseDeliverTx>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/tendermint.abci.ABCIApplication/DeliverTx",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCIApplication", "DeliverTx"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn check_tx(
            &mut self,
            request: impl tonic::IntoRequest<::tendermint_proto::abci::RequestCheckTx>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseCheckTx>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/tendermint.abci.ABCIApplication/CheckTx",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCIApplication", "CheckTx"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn query(
            &mut self,
            request: impl tonic::IntoRequest<::tendermint_proto::abci::RequestQuery>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseQuery>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/tendermint.abci.ABCIApplication/Query",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCIApplication", "Query"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn commit(
            &mut self,
            request: impl tonic::IntoRequest<::tendermint_proto::abci::RequestCommit>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseCommit>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/tendermint.abci.ABCIApplication/Commit",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCIApplication", "Commit"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn init_chain(
            &mut self,
            request: impl tonic::IntoRequest<::tendermint_proto::abci::RequestInitChain>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseInitChain>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/tendermint.abci.ABCIApplication/InitChain",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCIApplication", "InitChain"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn begin_block(
            &mut self,
            request: impl tonic::IntoRequest<::tendermint_proto::abci::RequestBeginBlock>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseBeginBlock>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/tendermint.abci.ABCIApplication/BeginBlock",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("tendermint.abci.ABCIApplication", "BeginBlock"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn end_block(
            &mut self,
            request: impl tonic::IntoRequest<::tendermint_proto::abci::RequestEndBlock>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseEndBlock>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/tendermint.abci.ABCIApplication/EndBlock",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("tendermint.abci.ABCIApplication", "EndBlock"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_snapshots(
            &mut self,
            request: impl tonic::IntoRequest<
                ::tendermint_proto::abci::RequestListSnapshots,
            >,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseListSnapshots>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
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
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("tendermint.abci.ABCIApplication", "ListSnapshots"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn offer_snapshot(
            &mut self,
            request: impl tonic::IntoRequest<
                ::tendermint_proto::abci::RequestOfferSnapshot,
            >,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseOfferSnapshot>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
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
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("tendermint.abci.ABCIApplication", "OfferSnapshot"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn load_snapshot_chunk(
            &mut self,
            request: impl tonic::IntoRequest<
                ::tendermint_proto::abci::RequestLoadSnapshotChunk,
            >,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseLoadSnapshotChunk>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
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
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "tendermint.abci.ABCIApplication",
                        "LoadSnapshotChunk",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn apply_snapshot_chunk(
            &mut self,
            request: impl tonic::IntoRequest<
                ::tendermint_proto::abci::RequestApplySnapshotChunk,
            >,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseApplySnapshotChunk>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
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
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "tendermint.abci.ABCIApplication",
                        "ApplySnapshotChunk",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn prepare_proposal(
            &mut self,
            request: impl tonic::IntoRequest<
                ::tendermint_proto::abci::RequestPrepareProposal,
            >,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponsePrepareProposal>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
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
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("tendermint.abci.ABCIApplication", "PrepareProposal"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn process_proposal(
            &mut self,
            request: impl tonic::IntoRequest<
                ::tendermint_proto::abci::RequestProcessProposal,
            >,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseProcessProposal>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
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
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("tendermint.abci.ABCIApplication", "ProcessProposal"),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "server")]
pub mod abci_application_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with AbciApplicationServer.
    #[async_trait]
    pub trait AbciApplication: Send + Sync + 'static {
        async fn echo(
            &self,
            request: tonic::Request<::tendermint_proto::abci::RequestEcho>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseEcho>,
            tonic::Status,
        >;
        async fn flush(
            &self,
            request: tonic::Request<::tendermint_proto::abci::RequestFlush>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseFlush>,
            tonic::Status,
        >;
        async fn info(
            &self,
            request: tonic::Request<::tendermint_proto::abci::RequestInfo>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseInfo>,
            tonic::Status,
        >;
        async fn deliver_tx(
            &self,
            request: tonic::Request<::tendermint_proto::abci::RequestDeliverTx>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseDeliverTx>,
            tonic::Status,
        >;
        async fn check_tx(
            &self,
            request: tonic::Request<::tendermint_proto::abci::RequestCheckTx>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseCheckTx>,
            tonic::Status,
        >;
        async fn query(
            &self,
            request: tonic::Request<::tendermint_proto::abci::RequestQuery>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseQuery>,
            tonic::Status,
        >;
        async fn commit(
            &self,
            request: tonic::Request<::tendermint_proto::abci::RequestCommit>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseCommit>,
            tonic::Status,
        >;
        async fn init_chain(
            &self,
            request: tonic::Request<::tendermint_proto::abci::RequestInitChain>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseInitChain>,
            tonic::Status,
        >;
        async fn begin_block(
            &self,
            request: tonic::Request<::tendermint_proto::abci::RequestBeginBlock>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseBeginBlock>,
            tonic::Status,
        >;
        async fn end_block(
            &self,
            request: tonic::Request<::tendermint_proto::abci::RequestEndBlock>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseEndBlock>,
            tonic::Status,
        >;
        async fn list_snapshots(
            &self,
            request: tonic::Request<::tendermint_proto::abci::RequestListSnapshots>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseListSnapshots>,
            tonic::Status,
        >;
        async fn offer_snapshot(
            &self,
            request: tonic::Request<::tendermint_proto::abci::RequestOfferSnapshot>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseOfferSnapshot>,
            tonic::Status,
        >;
        async fn load_snapshot_chunk(
            &self,
            request: tonic::Request<::tendermint_proto::abci::RequestLoadSnapshotChunk>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseLoadSnapshotChunk>,
            tonic::Status,
        >;
        async fn apply_snapshot_chunk(
            &self,
            request: tonic::Request<::tendermint_proto::abci::RequestApplySnapshotChunk>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseApplySnapshotChunk>,
            tonic::Status,
        >;
        async fn prepare_proposal(
            &self,
            request: tonic::Request<::tendermint_proto::abci::RequestPrepareProposal>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponsePrepareProposal>,
            tonic::Status,
        >;
        async fn process_proposal(
            &self,
            request: tonic::Request<::tendermint_proto::abci::RequestProcessProposal>,
        ) -> std::result::Result<
            tonic::Response<::tendermint_proto::abci::ResponseProcessProposal>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct AbciApplicationServer<T: AbciApplication> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: AbciApplication> AbciApplicationServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for AbciApplicationServer<T>
    where
        T: AbciApplication,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/tendermint.abci.ABCIApplication/Echo" => {
                    #[allow(non_camel_case_types)]
                    struct EchoSvc<T: AbciApplication>(pub Arc<T>);
                    impl<
                        T: AbciApplication,
                    > tonic::server::UnaryService<::tendermint_proto::abci::RequestEcho>
                    for EchoSvc<T> {
                        type Response = ::tendermint_proto::abci::ResponseEcho;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::tendermint_proto::abci::RequestEcho,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).echo(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = EchoSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCIApplication/Flush" => {
                    #[allow(non_camel_case_types)]
                    struct FlushSvc<T: AbciApplication>(pub Arc<T>);
                    impl<
                        T: AbciApplication,
                    > tonic::server::UnaryService<::tendermint_proto::abci::RequestFlush>
                    for FlushSvc<T> {
                        type Response = ::tendermint_proto::abci::ResponseFlush;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::tendermint_proto::abci::RequestFlush,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).flush(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = FlushSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCIApplication/Info" => {
                    #[allow(non_camel_case_types)]
                    struct InfoSvc<T: AbciApplication>(pub Arc<T>);
                    impl<
                        T: AbciApplication,
                    > tonic::server::UnaryService<::tendermint_proto::abci::RequestInfo>
                    for InfoSvc<T> {
                        type Response = ::tendermint_proto::abci::ResponseInfo;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::tendermint_proto::abci::RequestInfo,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).info(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = InfoSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCIApplication/DeliverTx" => {
                    #[allow(non_camel_case_types)]
                    struct DeliverTxSvc<T: AbciApplication>(pub Arc<T>);
                    impl<
                        T: AbciApplication,
                    > tonic::server::UnaryService<
                        ::tendermint_proto::abci::RequestDeliverTx,
                    > for DeliverTxSvc<T> {
                        type Response = ::tendermint_proto::abci::ResponseDeliverTx;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::tendermint_proto::abci::RequestDeliverTx,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).deliver_tx(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeliverTxSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCIApplication/CheckTx" => {
                    #[allow(non_camel_case_types)]
                    struct CheckTxSvc<T: AbciApplication>(pub Arc<T>);
                    impl<
                        T: AbciApplication,
                    > tonic::server::UnaryService<
                        ::tendermint_proto::abci::RequestCheckTx,
                    > for CheckTxSvc<T> {
                        type Response = ::tendermint_proto::abci::ResponseCheckTx;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::tendermint_proto::abci::RequestCheckTx,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).check_tx(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CheckTxSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCIApplication/Query" => {
                    #[allow(non_camel_case_types)]
                    struct QuerySvc<T: AbciApplication>(pub Arc<T>);
                    impl<
                        T: AbciApplication,
                    > tonic::server::UnaryService<::tendermint_proto::abci::RequestQuery>
                    for QuerySvc<T> {
                        type Response = ::tendermint_proto::abci::ResponseQuery;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::tendermint_proto::abci::RequestQuery,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).query(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = QuerySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCIApplication/Commit" => {
                    #[allow(non_camel_case_types)]
                    struct CommitSvc<T: AbciApplication>(pub Arc<T>);
                    impl<
                        T: AbciApplication,
                    > tonic::server::UnaryService<
                        ::tendermint_proto::abci::RequestCommit,
                    > for CommitSvc<T> {
                        type Response = ::tendermint_proto::abci::ResponseCommit;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::tendermint_proto::abci::RequestCommit,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).commit(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CommitSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCIApplication/InitChain" => {
                    #[allow(non_camel_case_types)]
                    struct InitChainSvc<T: AbciApplication>(pub Arc<T>);
                    impl<
                        T: AbciApplication,
                    > tonic::server::UnaryService<
                        ::tendermint_proto::abci::RequestInitChain,
                    > for InitChainSvc<T> {
                        type Response = ::tendermint_proto::abci::ResponseInitChain;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::tendermint_proto::abci::RequestInitChain,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).init_chain(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = InitChainSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCIApplication/BeginBlock" => {
                    #[allow(non_camel_case_types)]
                    struct BeginBlockSvc<T: AbciApplication>(pub Arc<T>);
                    impl<
                        T: AbciApplication,
                    > tonic::server::UnaryService<
                        ::tendermint_proto::abci::RequestBeginBlock,
                    > for BeginBlockSvc<T> {
                        type Response = ::tendermint_proto::abci::ResponseBeginBlock;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::tendermint_proto::abci::RequestBeginBlock,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).begin_block(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = BeginBlockSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCIApplication/EndBlock" => {
                    #[allow(non_camel_case_types)]
                    struct EndBlockSvc<T: AbciApplication>(pub Arc<T>);
                    impl<
                        T: AbciApplication,
                    > tonic::server::UnaryService<
                        ::tendermint_proto::abci::RequestEndBlock,
                    > for EndBlockSvc<T> {
                        type Response = ::tendermint_proto::abci::ResponseEndBlock;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::tendermint_proto::abci::RequestEndBlock,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).end_block(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = EndBlockSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCIApplication/ListSnapshots" => {
                    #[allow(non_camel_case_types)]
                    struct ListSnapshotsSvc<T: AbciApplication>(pub Arc<T>);
                    impl<
                        T: AbciApplication,
                    > tonic::server::UnaryService<
                        ::tendermint_proto::abci::RequestListSnapshots,
                    > for ListSnapshotsSvc<T> {
                        type Response = ::tendermint_proto::abci::ResponseListSnapshots;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::tendermint_proto::abci::RequestListSnapshots,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).list_snapshots(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListSnapshotsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCIApplication/OfferSnapshot" => {
                    #[allow(non_camel_case_types)]
                    struct OfferSnapshotSvc<T: AbciApplication>(pub Arc<T>);
                    impl<
                        T: AbciApplication,
                    > tonic::server::UnaryService<
                        ::tendermint_proto::abci::RequestOfferSnapshot,
                    > for OfferSnapshotSvc<T> {
                        type Response = ::tendermint_proto::abci::ResponseOfferSnapshot;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::tendermint_proto::abci::RequestOfferSnapshot,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).offer_snapshot(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = OfferSnapshotSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCIApplication/LoadSnapshotChunk" => {
                    #[allow(non_camel_case_types)]
                    struct LoadSnapshotChunkSvc<T: AbciApplication>(pub Arc<T>);
                    impl<
                        T: AbciApplication,
                    > tonic::server::UnaryService<
                        ::tendermint_proto::abci::RequestLoadSnapshotChunk,
                    > for LoadSnapshotChunkSvc<T> {
                        type Response = ::tendermint_proto::abci::ResponseLoadSnapshotChunk;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::tendermint_proto::abci::RequestLoadSnapshotChunk,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).load_snapshot_chunk(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = LoadSnapshotChunkSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCIApplication/ApplySnapshotChunk" => {
                    #[allow(non_camel_case_types)]
                    struct ApplySnapshotChunkSvc<T: AbciApplication>(pub Arc<T>);
                    impl<
                        T: AbciApplication,
                    > tonic::server::UnaryService<
                        ::tendermint_proto::abci::RequestApplySnapshotChunk,
                    > for ApplySnapshotChunkSvc<T> {
                        type Response = ::tendermint_proto::abci::ResponseApplySnapshotChunk;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::tendermint_proto::abci::RequestApplySnapshotChunk,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).apply_snapshot_chunk(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ApplySnapshotChunkSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCIApplication/PrepareProposal" => {
                    #[allow(non_camel_case_types)]
                    struct PrepareProposalSvc<T: AbciApplication>(pub Arc<T>);
                    impl<
                        T: AbciApplication,
                    > tonic::server::UnaryService<
                        ::tendermint_proto::abci::RequestPrepareProposal,
                    > for PrepareProposalSvc<T> {
                        type Response = ::tendermint_proto::abci::ResponsePrepareProposal;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::tendermint_proto::abci::RequestPrepareProposal,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).prepare_proposal(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PrepareProposalSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCIApplication/ProcessProposal" => {
                    #[allow(non_camel_case_types)]
                    struct ProcessProposalSvc<T: AbciApplication>(pub Arc<T>);
                    impl<
                        T: AbciApplication,
                    > tonic::server::UnaryService<
                        ::tendermint_proto::abci::RequestProcessProposal,
                    > for ProcessProposalSvc<T> {
                        type Response = ::tendermint_proto::abci::ResponseProcessProposal;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::tendermint_proto::abci::RequestProcessProposal,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).process_proposal(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ProcessProposalSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: AbciApplication> Clone for AbciApplicationServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: AbciApplication> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: AbciApplication> tonic::server::NamedService for AbciApplicationServer<T> {
        const NAME: &'static str = "tendermint.abci.ABCIApplication";
    }
}
