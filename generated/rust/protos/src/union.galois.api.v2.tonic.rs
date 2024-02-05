// @generated
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod union_prover_api_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::{http::Uri, *};
    ///
    #[derive(Debug, Clone)]
    pub struct UnionProverApiClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl UnionProverApiClient<tonic::transport::Channel> {
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
    impl<T> UnionProverApiClient<T>
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
        ) -> UnionProverApiClient<InterceptedService<T, F>>
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
            UnionProverApiClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn prove(
            &mut self,
            request: impl tonic::IntoRequest<super::ProveRequest>,
        ) -> std::result::Result<tonic::Response<super::ProveResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/union.galois.api.v2.UnionProverAPI/Prove");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "union.galois.api.v2.UnionProverAPI",
                "Prove",
            ));
            self.inner.unary(req, path, codec).await
        }
        ///
        pub async fn verify(
            &mut self,
            request: impl tonic::IntoRequest<super::VerifyRequest>,
        ) -> std::result::Result<tonic::Response<super::VerifyResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/union.galois.api.v2.UnionProverAPI/Verify");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "union.galois.api.v2.UnionProverAPI",
                "Verify",
            ));
            self.inner.unary(req, path, codec).await
        }
        ///
        pub async fn generate_contract(
            &mut self,
            request: impl tonic::IntoRequest<super::GenerateContractRequest>,
        ) -> std::result::Result<tonic::Response<super::GenerateContractResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/union.galois.api.v2.UnionProverAPI/GenerateContract",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "union.galois.api.v2.UnionProverAPI",
                "GenerateContract",
            ));
            self.inner.unary(req, path, codec).await
        }
        ///
        pub async fn query_stats(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryStatsRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryStatsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/union.galois.api.v2.UnionProverAPI/QueryStats",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "union.galois.api.v2.UnionProverAPI",
                "QueryStats",
            ));
            self.inner.unary(req, path, codec).await
        }
        ///
        pub async fn poll(
            &mut self,
            request: impl tonic::IntoRequest<super::PollRequest>,
        ) -> std::result::Result<tonic::Response<super::PollResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/union.galois.api.v2.UnionProverAPI/Poll");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "union.galois.api.v2.UnionProverAPI",
                "Poll",
            ));
            self.inner.unary(req, path, codec).await
        }
    }
}
