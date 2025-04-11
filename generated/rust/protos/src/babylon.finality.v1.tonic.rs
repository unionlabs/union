// @generated
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod query_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::{http::Uri, *};
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
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
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
        pub async fn params(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryParamsRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryParamsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/babylon.finality.v1.Query/Params");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("babylon.finality.v1.Query", "Params"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn active_finality_providers_at_height(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryActiveFinalityProvidersAtHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryActiveFinalityProvidersAtHeightResponse>,
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
                "/babylon.finality.v1.Query/ActiveFinalityProvidersAtHeight",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.finality.v1.Query",
                "ActiveFinalityProvidersAtHeight",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn finality_provider_power_at_height(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryFinalityProviderPowerAtHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryFinalityProviderPowerAtHeightResponse>,
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
                "/babylon.finality.v1.Query/FinalityProviderPowerAtHeight",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.finality.v1.Query",
                "FinalityProviderPowerAtHeight",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn finality_provider_current_power(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryFinalityProviderCurrentPowerRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryFinalityProviderCurrentPowerResponse>,
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
                "/babylon.finality.v1.Query/FinalityProviderCurrentPower",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.finality.v1.Query",
                "FinalityProviderCurrentPower",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn activated_height(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryActivatedHeightRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryActivatedHeightResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/babylon.finality.v1.Query/ActivatedHeight");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.finality.v1.Query",
                "ActivatedHeight",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_public_randomness(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryListPublicRandomnessRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryListPublicRandomnessResponse>,
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
                "/babylon.finality.v1.Query/ListPublicRandomness",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.finality.v1.Query",
                "ListPublicRandomness",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_pub_rand_commit(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryListPubRandCommitRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryListPubRandCommitResponse>,
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
                "/babylon.finality.v1.Query/ListPubRandCommit",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.finality.v1.Query",
                "ListPubRandCommit",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn block(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryBlockRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryBlockResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/babylon.finality.v1.Query/Block");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("babylon.finality.v1.Query", "Block"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_blocks(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryListBlocksRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryListBlocksResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/babylon.finality.v1.Query/ListBlocks");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("babylon.finality.v1.Query", "ListBlocks"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn votes_at_height(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryVotesAtHeightRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryVotesAtHeightResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/babylon.finality.v1.Query/VotesAtHeight");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.finality.v1.Query",
                "VotesAtHeight",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn evidence(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryEvidenceRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryEvidenceResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/babylon.finality.v1.Query/Evidence");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("babylon.finality.v1.Query", "Evidence"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_evidences(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryListEvidencesRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryListEvidencesResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/babylon.finality.v1.Query/ListEvidences");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.finality.v1.Query",
                "ListEvidences",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn signing_info(
            &mut self,
            request: impl tonic::IntoRequest<super::QuerySigningInfoRequest>,
        ) -> std::result::Result<tonic::Response<super::QuerySigningInfoResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/babylon.finality.v1.Query/SigningInfo");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("babylon.finality.v1.Query", "SigningInfo"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn signing_infos(
            &mut self,
            request: impl tonic::IntoRequest<super::QuerySigningInfosRequest>,
        ) -> std::result::Result<tonic::Response<super::QuerySigningInfosResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/babylon.finality.v1.Query/SigningInfos");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("babylon.finality.v1.Query", "SigningInfos"));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod msg_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::{http::Uri, *};
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
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> MsgClient<InterceptedService<T, F>>
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
        pub async fn commit_pub_rand_list(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgCommitPubRandList>,
        ) -> std::result::Result<tonic::Response<super::MsgCommitPubRandListResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/babylon.finality.v1.Msg/CommitPubRandList");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.finality.v1.Msg",
                "CommitPubRandList",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn add_finality_sig(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgAddFinalitySig>,
        ) -> std::result::Result<tonic::Response<super::MsgAddFinalitySigResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/babylon.finality.v1.Msg/AddFinalitySig");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("babylon.finality.v1.Msg", "AddFinalitySig"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_params(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgUpdateParams>,
        ) -> std::result::Result<tonic::Response<super::MsgUpdateParamsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/babylon.finality.v1.Msg/UpdateParams");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("babylon.finality.v1.Msg", "UpdateParams"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn unjail_finality_provider(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgUnjailFinalityProvider>,
        ) -> std::result::Result<
            tonic::Response<super::MsgUnjailFinalityProviderResponse>,
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
                "/babylon.finality.v1.Msg/UnjailFinalityProvider",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.finality.v1.Msg",
                "UnjailFinalityProvider",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn resume_finality_proposal(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgResumeFinalityProposal>,
        ) -> std::result::Result<
            tonic::Response<super::MsgResumeFinalityProposalResponse>,
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
                "/babylon.finality.v1.Msg/ResumeFinalityProposal",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.finality.v1.Msg",
                "ResumeFinalityProposal",
            ));
            self.inner.unary(req, path, codec).await
        }
    }
}
