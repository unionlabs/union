// @generated
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod reflection_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::{http::Uri, *};
    #[derive(Debug, Clone)]
    pub struct ReflectionServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ReflectionServiceClient<tonic::transport::Channel> {
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
    impl<T> ReflectionServiceClient<T>
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
        ) -> ReflectionServiceClient<InterceptedService<T, F>>
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
            ReflectionServiceClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn get_authn_descriptor(
            &mut self,
            request: impl tonic::IntoRequest<super::GetAuthnDescriptorRequest>,
        ) -> Result<tonic::Response<super::GetAuthnDescriptorResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.base.reflection.v2alpha1.ReflectionService/GetAuthnDescriptor",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_chain_descriptor(
            &mut self,
            request: impl tonic::IntoRequest<super::GetChainDescriptorRequest>,
        ) -> Result<tonic::Response<super::GetChainDescriptorResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.base.reflection.v2alpha1.ReflectionService/GetChainDescriptor",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_codec_descriptor(
            &mut self,
            request: impl tonic::IntoRequest<super::GetCodecDescriptorRequest>,
        ) -> Result<tonic::Response<super::GetCodecDescriptorResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.base.reflection.v2alpha1.ReflectionService/GetCodecDescriptor",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_configuration_descriptor(
            &mut self,
            request: impl tonic::IntoRequest<super::GetConfigurationDescriptorRequest>,
        ) -> Result<tonic::Response<super::GetConfigurationDescriptorResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.base.reflection.v2alpha1.ReflectionService/GetConfigurationDescriptor",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_query_services_descriptor(
            &mut self,
            request: impl tonic::IntoRequest<super::GetQueryServicesDescriptorRequest>,
        ) -> Result<tonic::Response<super::GetQueryServicesDescriptorResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.base.reflection.v2alpha1.ReflectionService/GetQueryServicesDescriptor",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_tx_descriptor(
            &mut self,
            request: impl tonic::IntoRequest<super::GetTxDescriptorRequest>,
        ) -> Result<tonic::Response<super::GetTxDescriptorResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.base.reflection.v2alpha1.ReflectionService/GetTxDescriptor",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "server")]
pub mod reflection_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with ReflectionServiceServer.
    #[async_trait]
    pub trait ReflectionService: Send + Sync + 'static {
        async fn get_authn_descriptor(
            &self,
            request: tonic::Request<super::GetAuthnDescriptorRequest>,
        ) -> Result<tonic::Response<super::GetAuthnDescriptorResponse>, tonic::Status>;
        async fn get_chain_descriptor(
            &self,
            request: tonic::Request<super::GetChainDescriptorRequest>,
        ) -> Result<tonic::Response<super::GetChainDescriptorResponse>, tonic::Status>;
        async fn get_codec_descriptor(
            &self,
            request: tonic::Request<super::GetCodecDescriptorRequest>,
        ) -> Result<tonic::Response<super::GetCodecDescriptorResponse>, tonic::Status>;
        async fn get_configuration_descriptor(
            &self,
            request: tonic::Request<super::GetConfigurationDescriptorRequest>,
        ) -> Result<tonic::Response<super::GetConfigurationDescriptorResponse>, tonic::Status>;
        async fn get_query_services_descriptor(
            &self,
            request: tonic::Request<super::GetQueryServicesDescriptorRequest>,
        ) -> Result<tonic::Response<super::GetQueryServicesDescriptorResponse>, tonic::Status>;
        async fn get_tx_descriptor(
            &self,
            request: tonic::Request<super::GetTxDescriptorRequest>,
        ) -> Result<tonic::Response<super::GetTxDescriptorResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct ReflectionServiceServer<T: ReflectionService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: ReflectionService> ReflectionServiceServer<T> {
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
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for ReflectionServiceServer<T>
    where
        T: ReflectionService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/cosmos.base.reflection.v2alpha1.ReflectionService/GetAuthnDescriptor" => {
                    #[allow(non_camel_case_types)]
                    struct GetAuthnDescriptorSvc<T: ReflectionService>(pub Arc<T>);
                    impl<T: ReflectionService>
                        tonic::server::UnaryService<super::GetAuthnDescriptorRequest>
                        for GetAuthnDescriptorSvc<T>
                    {
                        type Response = super::GetAuthnDescriptorResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetAuthnDescriptorRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_authn_descriptor(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetAuthnDescriptorSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.base.reflection.v2alpha1.ReflectionService/GetChainDescriptor" => {
                    #[allow(non_camel_case_types)]
                    struct GetChainDescriptorSvc<T: ReflectionService>(pub Arc<T>);
                    impl<T: ReflectionService>
                        tonic::server::UnaryService<super::GetChainDescriptorRequest>
                        for GetChainDescriptorSvc<T>
                    {
                        type Response = super::GetChainDescriptorResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetChainDescriptorRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_chain_descriptor(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetChainDescriptorSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.base.reflection.v2alpha1.ReflectionService/GetCodecDescriptor" => {
                    #[allow(non_camel_case_types)]
                    struct GetCodecDescriptorSvc<T: ReflectionService>(pub Arc<T>);
                    impl<T: ReflectionService>
                        tonic::server::UnaryService<super::GetCodecDescriptorRequest>
                        for GetCodecDescriptorSvc<T>
                    {
                        type Response = super::GetCodecDescriptorResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetCodecDescriptorRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_codec_descriptor(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetCodecDescriptorSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.base.reflection.v2alpha1.ReflectionService/GetConfigurationDescriptor" => {
                    #[allow(non_camel_case_types)]
                    struct GetConfigurationDescriptorSvc<T: ReflectionService>(pub Arc<T>);
                    impl<T: ReflectionService>
                        tonic::server::UnaryService<super::GetConfigurationDescriptorRequest>
                        for GetConfigurationDescriptorSvc<T>
                    {
                        type Response = super::GetConfigurationDescriptorResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetConfigurationDescriptorRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).get_configuration_descriptor(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetConfigurationDescriptorSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.base.reflection.v2alpha1.ReflectionService/GetQueryServicesDescriptor" => {
                    #[allow(non_camel_case_types)]
                    struct GetQueryServicesDescriptorSvc<T: ReflectionService>(pub Arc<T>);
                    impl<T: ReflectionService>
                        tonic::server::UnaryService<super::GetQueryServicesDescriptorRequest>
                        for GetQueryServicesDescriptorSvc<T>
                    {
                        type Response = super::GetQueryServicesDescriptorResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetQueryServicesDescriptorRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_query_services_descriptor(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetQueryServicesDescriptorSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.base.reflection.v2alpha1.ReflectionService/GetTxDescriptor" => {
                    #[allow(non_camel_case_types)]
                    struct GetTxDescriptorSvc<T: ReflectionService>(pub Arc<T>);
                    impl<T: ReflectionService>
                        tonic::server::UnaryService<super::GetTxDescriptorRequest>
                        for GetTxDescriptorSvc<T>
                    {
                        type Response = super::GetTxDescriptorResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetTxDescriptorRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_tx_descriptor(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetTxDescriptorSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: ReflectionService> Clone for ReflectionServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: ReflectionService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: ReflectionService> tonic::server::NamedService for ReflectionServiceServer<T> {
        const NAME: &'static str = "cosmos.base.reflection.v2alpha1.ReflectionService";
    }
}
