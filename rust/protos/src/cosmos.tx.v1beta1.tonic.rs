// @generated
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct ServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ServiceClient<tonic::transport::Channel> {
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
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
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
        pub async fn simulate(
            &mut self,
            request: impl tonic::IntoRequest<super::SimulateRequest>,
        ) -> Result<tonic::Response<super::SimulateResponse>, tonic::Status> {
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
                "/cosmos.tx.v1beta1.Service/Simulate",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_tx(
            &mut self,
            request: impl tonic::IntoRequest<super::GetTxRequest>,
        ) -> Result<tonic::Response<super::GetTxResponse>, tonic::Status> {
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
                "/cosmos.tx.v1beta1.Service/GetTx",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn broadcast_tx(
            &mut self,
            request: impl tonic::IntoRequest<super::BroadcastTxRequest>,
        ) -> Result<tonic::Response<super::BroadcastTxResponse>, tonic::Status> {
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
                "/cosmos.tx.v1beta1.Service/BroadcastTx",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_txs_event(
            &mut self,
            request: impl tonic::IntoRequest<super::GetTxsEventRequest>,
        ) -> Result<tonic::Response<super::GetTxsEventResponse>, tonic::Status> {
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
                "/cosmos.tx.v1beta1.Service/GetTxsEvent",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_block_with_txs(
            &mut self,
            request: impl tonic::IntoRequest<super::GetBlockWithTxsRequest>,
        ) -> Result<tonic::Response<super::GetBlockWithTxsResponse>, tonic::Status> {
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
                "/cosmos.tx.v1beta1.Service/GetBlockWithTxs",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn tx_decode(
            &mut self,
            request: impl tonic::IntoRequest<super::TxDecodeRequest>,
        ) -> Result<tonic::Response<super::TxDecodeResponse>, tonic::Status> {
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
                "/cosmos.tx.v1beta1.Service/TxDecode",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn tx_encode(
            &mut self,
            request: impl tonic::IntoRequest<super::TxEncodeRequest>,
        ) -> Result<tonic::Response<super::TxEncodeResponse>, tonic::Status> {
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
                "/cosmos.tx.v1beta1.Service/TxEncode",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn tx_encode_amino(
            &mut self,
            request: impl tonic::IntoRequest<super::TxEncodeAminoRequest>,
        ) -> Result<tonic::Response<super::TxEncodeAminoResponse>, tonic::Status> {
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
                "/cosmos.tx.v1beta1.Service/TxEncodeAmino",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn tx_decode_amino(
            &mut self,
            request: impl tonic::IntoRequest<super::TxDecodeAminoRequest>,
        ) -> Result<tonic::Response<super::TxDecodeAminoResponse>, tonic::Status> {
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
                "/cosmos.tx.v1beta1.Service/TxDecodeAmino",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "server")]
pub mod service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with ServiceServer.
    #[async_trait]
    pub trait Service: Send + Sync + 'static {
        async fn simulate(
            &self,
            request: tonic::Request<super::SimulateRequest>,
        ) -> Result<tonic::Response<super::SimulateResponse>, tonic::Status>;
        async fn get_tx(
            &self,
            request: tonic::Request<super::GetTxRequest>,
        ) -> Result<tonic::Response<super::GetTxResponse>, tonic::Status>;
        async fn broadcast_tx(
            &self,
            request: tonic::Request<super::BroadcastTxRequest>,
        ) -> Result<tonic::Response<super::BroadcastTxResponse>, tonic::Status>;
        async fn get_txs_event(
            &self,
            request: tonic::Request<super::GetTxsEventRequest>,
        ) -> Result<tonic::Response<super::GetTxsEventResponse>, tonic::Status>;
        async fn get_block_with_txs(
            &self,
            request: tonic::Request<super::GetBlockWithTxsRequest>,
        ) -> Result<tonic::Response<super::GetBlockWithTxsResponse>, tonic::Status>;
        async fn tx_decode(
            &self,
            request: tonic::Request<super::TxDecodeRequest>,
        ) -> Result<tonic::Response<super::TxDecodeResponse>, tonic::Status>;
        async fn tx_encode(
            &self,
            request: tonic::Request<super::TxEncodeRequest>,
        ) -> Result<tonic::Response<super::TxEncodeResponse>, tonic::Status>;
        async fn tx_encode_amino(
            &self,
            request: tonic::Request<super::TxEncodeAminoRequest>,
        ) -> Result<tonic::Response<super::TxEncodeAminoResponse>, tonic::Status>;
        async fn tx_decode_amino(
            &self,
            request: tonic::Request<super::TxDecodeAminoRequest>,
        ) -> Result<tonic::Response<super::TxDecodeAminoResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct ServiceServer<T: Service> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Service> ServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for ServiceServer<T>
    where
        T: Service,
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
                "/cosmos.tx.v1beta1.Service/Simulate" => {
                    #[allow(non_camel_case_types)]
                    struct SimulateSvc<T: Service>(pub Arc<T>);
                    impl<T: Service> tonic::server::UnaryService<super::SimulateRequest>
                    for SimulateSvc<T> {
                        type Response = super::SimulateResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SimulateRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).simulate(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SimulateSvc(inner);
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
                "/cosmos.tx.v1beta1.Service/GetTx" => {
                    #[allow(non_camel_case_types)]
                    struct GetTxSvc<T: Service>(pub Arc<T>);
                    impl<T: Service> tonic::server::UnaryService<super::GetTxRequest>
                    for GetTxSvc<T> {
                        type Response = super::GetTxResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetTxRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_tx(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetTxSvc(inner);
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
                "/cosmos.tx.v1beta1.Service/BroadcastTx" => {
                    #[allow(non_camel_case_types)]
                    struct BroadcastTxSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<super::BroadcastTxRequest>
                    for BroadcastTxSvc<T> {
                        type Response = super::BroadcastTxResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BroadcastTxRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).broadcast_tx(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = BroadcastTxSvc(inner);
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
                "/cosmos.tx.v1beta1.Service/GetTxsEvent" => {
                    #[allow(non_camel_case_types)]
                    struct GetTxsEventSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<super::GetTxsEventRequest>
                    for GetTxsEventSvc<T> {
                        type Response = super::GetTxsEventResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetTxsEventRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_txs_event(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetTxsEventSvc(inner);
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
                "/cosmos.tx.v1beta1.Service/GetBlockWithTxs" => {
                    #[allow(non_camel_case_types)]
                    struct GetBlockWithTxsSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<super::GetBlockWithTxsRequest>
                    for GetBlockWithTxsSvc<T> {
                        type Response = super::GetBlockWithTxsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetBlockWithTxsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_block_with_txs(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetBlockWithTxsSvc(inner);
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
                "/cosmos.tx.v1beta1.Service/TxDecode" => {
                    #[allow(non_camel_case_types)]
                    struct TxDecodeSvc<T: Service>(pub Arc<T>);
                    impl<T: Service> tonic::server::UnaryService<super::TxDecodeRequest>
                    for TxDecodeSvc<T> {
                        type Response = super::TxDecodeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TxDecodeRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).tx_decode(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TxDecodeSvc(inner);
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
                "/cosmos.tx.v1beta1.Service/TxEncode" => {
                    #[allow(non_camel_case_types)]
                    struct TxEncodeSvc<T: Service>(pub Arc<T>);
                    impl<T: Service> tonic::server::UnaryService<super::TxEncodeRequest>
                    for TxEncodeSvc<T> {
                        type Response = super::TxEncodeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TxEncodeRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).tx_encode(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TxEncodeSvc(inner);
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
                "/cosmos.tx.v1beta1.Service/TxEncodeAmino" => {
                    #[allow(non_camel_case_types)]
                    struct TxEncodeAminoSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<super::TxEncodeAminoRequest>
                    for TxEncodeAminoSvc<T> {
                        type Response = super::TxEncodeAminoResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TxEncodeAminoRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).tx_encode_amino(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TxEncodeAminoSvc(inner);
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
                "/cosmos.tx.v1beta1.Service/TxDecodeAmino" => {
                    #[allow(non_camel_case_types)]
                    struct TxDecodeAminoSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<super::TxDecodeAminoRequest>
                    for TxDecodeAminoSvc<T> {
                        type Response = super::TxDecodeAminoResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TxDecodeAminoRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).tx_decode_amino(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TxDecodeAminoSvc(inner);
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
    impl<T: Service> Clone for ServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Service> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Service> tonic::server::NamedService for ServiceServer<T> {
        const NAME: &'static str = "cosmos.tx.v1beta1.Service";
    }
}
