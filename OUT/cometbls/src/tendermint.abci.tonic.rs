// @generated
/// Generated client implementations.
pub mod abci_application_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    ///
    #[derive(Debug, Clone)]
    pub struct AbciApplicationClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl AbciApplicationClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
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
        ///
        pub async fn echo(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestEcho>,
        ) -> Result<tonic::Response<super::ResponseEcho>, tonic::Status> {
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
            self.inner.unary(request.into_request(), path, codec).await
        }
        ///
        pub async fn flush(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestFlush>,
        ) -> Result<tonic::Response<super::ResponseFlush>, tonic::Status> {
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
            self.inner.unary(request.into_request(), path, codec).await
        }
        ///
        pub async fn info(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestInfo>,
        ) -> Result<tonic::Response<super::ResponseInfo>, tonic::Status> {
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
            self.inner.unary(request.into_request(), path, codec).await
        }
        ///
        pub async fn deliver_tx(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestDeliverTx>,
        ) -> Result<tonic::Response<super::ResponseDeliverTx>, tonic::Status> {
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
            self.inner.unary(request.into_request(), path, codec).await
        }
        ///
        pub async fn check_tx(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestCheckTx>,
        ) -> Result<tonic::Response<super::ResponseCheckTx>, tonic::Status> {
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
            self.inner.unary(request.into_request(), path, codec).await
        }
        ///
        pub async fn query(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestQuery>,
        ) -> Result<tonic::Response<super::ResponseQuery>, tonic::Status> {
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
            self.inner.unary(request.into_request(), path, codec).await
        }
        ///
        pub async fn commit(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestCommit>,
        ) -> Result<tonic::Response<super::ResponseCommit>, tonic::Status> {
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
            self.inner.unary(request.into_request(), path, codec).await
        }
        ///
        pub async fn init_chain(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestInitChain>,
        ) -> Result<tonic::Response<super::ResponseInitChain>, tonic::Status> {
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
            self.inner.unary(request.into_request(), path, codec).await
        }
        ///
        pub async fn begin_block(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestBeginBlock>,
        ) -> Result<tonic::Response<super::ResponseBeginBlock>, tonic::Status> {
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
            self.inner.unary(request.into_request(), path, codec).await
        }
        ///
        pub async fn end_block(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestEndBlock>,
        ) -> Result<tonic::Response<super::ResponseEndBlock>, tonic::Status> {
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
            self.inner.unary(request.into_request(), path, codec).await
        }
        ///
        pub async fn list_snapshots(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestListSnapshots>,
        ) -> Result<tonic::Response<super::ResponseListSnapshots>, tonic::Status> {
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
            self.inner.unary(request.into_request(), path, codec).await
        }
        ///
        pub async fn offer_snapshot(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestOfferSnapshot>,
        ) -> Result<tonic::Response<super::ResponseOfferSnapshot>, tonic::Status> {
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
            self.inner.unary(request.into_request(), path, codec).await
        }
        ///
        pub async fn load_snapshot_chunk(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestLoadSnapshotChunk>,
        ) -> Result<tonic::Response<super::ResponseLoadSnapshotChunk>, tonic::Status> {
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
            self.inner.unary(request.into_request(), path, codec).await
        }
        ///
        pub async fn apply_snapshot_chunk(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestApplySnapshotChunk>,
        ) -> Result<tonic::Response<super::ResponseApplySnapshotChunk>, tonic::Status> {
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
            self.inner.unary(request.into_request(), path, codec).await
        }
        ///
        pub async fn prepare_proposal(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestPrepareProposal>,
        ) -> Result<tonic::Response<super::ResponsePrepareProposal>, tonic::Status> {
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
            self.inner.unary(request.into_request(), path, codec).await
        }
        ///
        pub async fn process_proposal(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestProcessProposal>,
        ) -> Result<tonic::Response<super::ResponseProcessProposal>, tonic::Status> {
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
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod abci_application_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with AbciApplicationServer.
    #[async_trait]
    pub trait AbciApplication: Send + Sync + 'static {
        ///
        async fn echo(
            &self,
            request: tonic::Request<super::RequestEcho>,
        ) -> Result<tonic::Response<super::ResponseEcho>, tonic::Status>;
        ///
        async fn flush(
            &self,
            request: tonic::Request<super::RequestFlush>,
        ) -> Result<tonic::Response<super::ResponseFlush>, tonic::Status>;
        ///
        async fn info(
            &self,
            request: tonic::Request<super::RequestInfo>,
        ) -> Result<tonic::Response<super::ResponseInfo>, tonic::Status>;
        ///
        async fn deliver_tx(
            &self,
            request: tonic::Request<super::RequestDeliverTx>,
        ) -> Result<tonic::Response<super::ResponseDeliverTx>, tonic::Status>;
        ///
        async fn check_tx(
            &self,
            request: tonic::Request<super::RequestCheckTx>,
        ) -> Result<tonic::Response<super::ResponseCheckTx>, tonic::Status>;
        ///
        async fn query(
            &self,
            request: tonic::Request<super::RequestQuery>,
        ) -> Result<tonic::Response<super::ResponseQuery>, tonic::Status>;
        ///
        async fn commit(
            &self,
            request: tonic::Request<super::RequestCommit>,
        ) -> Result<tonic::Response<super::ResponseCommit>, tonic::Status>;
        ///
        async fn init_chain(
            &self,
            request: tonic::Request<super::RequestInitChain>,
        ) -> Result<tonic::Response<super::ResponseInitChain>, tonic::Status>;
        ///
        async fn begin_block(
            &self,
            request: tonic::Request<super::RequestBeginBlock>,
        ) -> Result<tonic::Response<super::ResponseBeginBlock>, tonic::Status>;
        ///
        async fn end_block(
            &self,
            request: tonic::Request<super::RequestEndBlock>,
        ) -> Result<tonic::Response<super::ResponseEndBlock>, tonic::Status>;
        ///
        async fn list_snapshots(
            &self,
            request: tonic::Request<super::RequestListSnapshots>,
        ) -> Result<tonic::Response<super::ResponseListSnapshots>, tonic::Status>;
        ///
        async fn offer_snapshot(
            &self,
            request: tonic::Request<super::RequestOfferSnapshot>,
        ) -> Result<tonic::Response<super::ResponseOfferSnapshot>, tonic::Status>;
        ///
        async fn load_snapshot_chunk(
            &self,
            request: tonic::Request<super::RequestLoadSnapshotChunk>,
        ) -> Result<tonic::Response<super::ResponseLoadSnapshotChunk>, tonic::Status>;
        ///
        async fn apply_snapshot_chunk(
            &self,
            request: tonic::Request<super::RequestApplySnapshotChunk>,
        ) -> Result<tonic::Response<super::ResponseApplySnapshotChunk>, tonic::Status>;
        ///
        async fn prepare_proposal(
            &self,
            request: tonic::Request<super::RequestPrepareProposal>,
        ) -> Result<tonic::Response<super::ResponsePrepareProposal>, tonic::Status>;
        ///
        async fn process_proposal(
            &self,
            request: tonic::Request<super::RequestProcessProposal>,
        ) -> Result<tonic::Response<super::ResponseProcessProposal>, tonic::Status>;
    }
    ///
    #[derive(Debug)]
    pub struct AbciApplicationServer<T: AbciApplication> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
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
        ) -> Poll<Result<(), Self::Error>> {
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
                    > tonic::server::UnaryService<super::RequestEcho> for EchoSvc<T> {
                        type Response = super::ResponseEcho;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestEcho>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).echo(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = EchoSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                    > tonic::server::UnaryService<super::RequestFlush> for FlushSvc<T> {
                        type Response = super::ResponseFlush;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestFlush>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).flush(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = FlushSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                    > tonic::server::UnaryService<super::RequestInfo> for InfoSvc<T> {
                        type Response = super::ResponseInfo;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestInfo>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).info(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = InfoSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                    > tonic::server::UnaryService<super::RequestDeliverTx>
                    for DeliverTxSvc<T> {
                        type Response = super::ResponseDeliverTx;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestDeliverTx>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).deliver_tx(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeliverTxSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                    > tonic::server::UnaryService<super::RequestCheckTx>
                    for CheckTxSvc<T> {
                        type Response = super::ResponseCheckTx;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestCheckTx>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).check_tx(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CheckTxSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                    > tonic::server::UnaryService<super::RequestQuery> for QuerySvc<T> {
                        type Response = super::ResponseQuery;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestQuery>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).query(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = QuerySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                    > tonic::server::UnaryService<super::RequestCommit>
                    for CommitSvc<T> {
                        type Response = super::ResponseCommit;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestCommit>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).commit(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CommitSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                    > tonic::server::UnaryService<super::RequestInitChain>
                    for InitChainSvc<T> {
                        type Response = super::ResponseInitChain;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestInitChain>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).init_chain(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = InitChainSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                    > tonic::server::UnaryService<super::RequestBeginBlock>
                    for BeginBlockSvc<T> {
                        type Response = super::ResponseBeginBlock;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestBeginBlock>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).begin_block(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = BeginBlockSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                    > tonic::server::UnaryService<super::RequestEndBlock>
                    for EndBlockSvc<T> {
                        type Response = super::ResponseEndBlock;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestEndBlock>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).end_block(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = EndBlockSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                    > tonic::server::UnaryService<super::RequestListSnapshots>
                    for ListSnapshotsSvc<T> {
                        type Response = super::ResponseListSnapshots;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestListSnapshots>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).list_snapshots(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListSnapshotsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                    > tonic::server::UnaryService<super::RequestOfferSnapshot>
                    for OfferSnapshotSvc<T> {
                        type Response = super::ResponseOfferSnapshot;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestOfferSnapshot>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).offer_snapshot(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = OfferSnapshotSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                    > tonic::server::UnaryService<super::RequestLoadSnapshotChunk>
                    for LoadSnapshotChunkSvc<T> {
                        type Response = super::ResponseLoadSnapshotChunk;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestLoadSnapshotChunk>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).load_snapshot_chunk(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = LoadSnapshotChunkSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                    > tonic::server::UnaryService<super::RequestApplySnapshotChunk>
                    for ApplySnapshotChunkSvc<T> {
                        type Response = super::ResponseApplySnapshotChunk;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestApplySnapshotChunk>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).apply_snapshot_chunk(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ApplySnapshotChunkSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                    > tonic::server::UnaryService<super::RequestPrepareProposal>
                    for PrepareProposalSvc<T> {
                        type Response = super::ResponsePrepareProposal;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestPrepareProposal>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).prepare_proposal(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PrepareProposalSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
                    > tonic::server::UnaryService<super::RequestProcessProposal>
                    for ProcessProposalSvc<T> {
                        type Response = super::ResponseProcessProposal;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestProcessProposal>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).process_proposal(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ProcessProposalSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
            }
        }
    }
    impl<T: AbciApplication> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
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
