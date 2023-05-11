/// QueryCodeIdsRequest is the request type for the Query/CodeIds RPC method.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCodeIdsRequest {
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "1")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageRequest,
    >,
}
/// QueryCodeIdsResponse is the response type for the Query/CodeIds RPC method.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCodeIdsResponse {
    #[prost(string, repeated, tag = "1")]
    pub code_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageResponse,
    >,
}
/// QueryCodeRequest is the request type for the Query/Code RPC method.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCodeRequest {
    #[prost(string, tag = "1")]
    pub code_id: ::prost::alloc::string::String,
}
/// QueryCodeResponse is the response type for the Query/Code RPC method.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCodeResponse {
    #[prost(bytes = "vec", tag = "1")]
    pub code: ::prost::alloc::vec::Vec<u8>,
}
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod query_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Query service for wasm module
    #[derive(Debug, Clone)]
    pub struct QueryClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl QueryClient<tonic::transport::Channel> {
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
    impl<T> QueryClient<T>
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
        ) -> QueryClient<InterceptedService<T, F>>
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
            QueryClient::new(InterceptedService::new(inner, interceptor))
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
        /// Get all Wasm code IDs
        pub async fn code_ids(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryCodeIdsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryCodeIdsResponse>,
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
                "/ibc.lightclients.wasm.v1.Query/CodeIds",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.lightclients.wasm.v1.Query", "CodeIds"));
            self.inner.unary(req, path, codec).await
        }
        /// Get Wasm code for given code ID
        pub async fn code(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryCodeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryCodeResponse>,
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
                "/ibc.lightclients.wasm.v1.Query/Code",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.lightclients.wasm.v1.Query", "Code"));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "server")]
pub mod query_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with QueryServer.
    #[async_trait]
    pub trait Query: Send + Sync + 'static {
        /// Get all Wasm code IDs
        async fn code_ids(
            &self,
            request: tonic::Request<super::QueryCodeIdsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryCodeIdsResponse>,
            tonic::Status,
        >;
        /// Get Wasm code for given code ID
        async fn code(
            &self,
            request: tonic::Request<super::QueryCodeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryCodeResponse>,
            tonic::Status,
        >;
    }
    /// Query service for wasm module
    #[derive(Debug)]
    pub struct QueryServer<T: Query> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Query> QueryServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for QueryServer<T>
    where
        T: Query,
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
                "/ibc.lightclients.wasm.v1.Query/CodeIds" => {
                    #[allow(non_camel_case_types)]
                    struct CodeIdsSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryCodeIdsRequest>
                    for CodeIdsSvc<T> {
                        type Response = super::QueryCodeIdsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryCodeIdsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).code_ids(request).await };
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
                        let method = CodeIdsSvc(inner);
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
                "/ibc.lightclients.wasm.v1.Query/Code" => {
                    #[allow(non_camel_case_types)]
                    struct CodeSvc<T: Query>(pub Arc<T>);
                    impl<T: Query> tonic::server::UnaryService<super::QueryCodeRequest>
                    for CodeSvc<T> {
                        type Response = super::QueryCodeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryCodeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).code(request).await };
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
                        let method = CodeSvc(inner);
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
    impl<T: Query> Clone for QueryServer<T> {
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
    impl<T: Query> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Query> tonic::server::NamedService for QueryServer<T> {
        const NAME: &'static str = "ibc.lightclients.wasm.v1.Query";
    }
}
/// MsgStoreCode defines the request type for the StoreCode rpc.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgStoreCode {
    #[prost(string, tag = "1")]
    pub signer: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "2")]
    pub code: ::prost::alloc::vec::Vec<u8>,
}
/// MsgStoreCodeResponse defines the response type for the StoreCode rpc
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgStoreCodeResponse {
    #[prost(bytes = "vec", tag = "1")]
    pub code_id: ::prost::alloc::vec::Vec<u8>,
}
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod msg_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Msg defines the ibc/08-wasm Msg service.
    #[derive(Debug, Clone)]
    pub struct MsgClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl MsgClient<tonic::transport::Channel> {
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
    impl<T> MsgClient<T>
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
        ) -> MsgClient<InterceptedService<T, F>>
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
            MsgClient::new(InterceptedService::new(inner, interceptor))
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
        /// StoreCode defines a rpc handler method for MsgStoreCode.
        pub async fn store_code(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgStoreCode>,
        ) -> std::result::Result<
            tonic::Response<super::MsgStoreCodeResponse>,
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
                "/ibc.lightclients.wasm.v1.Msg/StoreCode",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.lightclients.wasm.v1.Msg", "StoreCode"));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "server")]
pub mod msg_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with MsgServer.
    #[async_trait]
    pub trait Msg: Send + Sync + 'static {
        /// StoreCode defines a rpc handler method for MsgStoreCode.
        async fn store_code(
            &self,
            request: tonic::Request<super::MsgStoreCode>,
        ) -> std::result::Result<
            tonic::Response<super::MsgStoreCodeResponse>,
            tonic::Status,
        >;
    }
    /// Msg defines the ibc/08-wasm Msg service.
    #[derive(Debug)]
    pub struct MsgServer<T: Msg> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Msg> MsgServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for MsgServer<T>
    where
        T: Msg,
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
                "/ibc.lightclients.wasm.v1.Msg/StoreCode" => {
                    #[allow(non_camel_case_types)]
                    struct StoreCodeSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgStoreCode>
                    for StoreCodeSvc<T> {
                        type Response = super::MsgStoreCodeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgStoreCode>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).store_code(request).await };
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
                        let method = StoreCodeSvc(inner);
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
    impl<T: Msg> Clone for MsgServer<T> {
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
    impl<T: Msg> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Msg> tonic::server::NamedService for MsgServer<T> {
        const NAME: &'static str = "ibc.lightclients.wasm.v1.Msg";
    }
}
/// Wasm light client's Client state
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    #[prost(bytes = "vec", tag = "1")]
    #[serde(with = "crate::base64")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    #[serde(with = "crate::base64")]
    pub code_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub latest_height: ::core::option::Option<
        super::super::super::core::client::v1::Height,
    >,
}
/// Wasm light client's ConsensusState
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusState {
    #[prost(bytes = "vec", tag = "1")]
    #[serde(with = "crate::base64")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    /// timestamp that corresponds to the block height in which the ConsensusState
    /// was stored.
    #[prost(uint64, tag = "2")]
    pub timestamp: u64,
}
/// Wasm light client Header
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
    #[prost(bytes = "vec", tag = "1")]
    #[serde(with = "crate::base64")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub height: ::core::option::Option<super::super::super::core::client::v1::Height>,
}
/// Wasm light client Misbehaviour
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Misbehaviour {
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
/// Wasm light client's keeper genesis state
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// uploaded light client wasm contracts
    #[prost(message, repeated, tag = "1")]
    pub contracts: ::prost::alloc::vec::Vec<GenesisContract>,
}
/// A contract's store key and code
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisContract {
    /// store key of metadata without clientID-prefix
    #[prost(bytes = "vec", tag = "1")]
    pub code_id_key: ::prost::alloc::vec::Vec<u8>,
    /// metadata value
    #[prost(bytes = "vec", tag = "2")]
    pub contract_code: ::prost::alloc::vec::Vec<u8>,
}
