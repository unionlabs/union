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
            let path = http::uri::PathAndQuery::from_static("/babylon.btcstaking.v1.Query/Params");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("babylon.btcstaking.v1.Query", "Params"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn params_versions(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryParamsVersionsRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryParamsVersionsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/babylon.btcstaking.v1.Query/ParamsVersions");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.btcstaking.v1.Query",
                "ParamsVersions",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn params_by_version(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryParamsByVersionRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryParamsByVersionResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/babylon.btcstaking.v1.Query/ParamsByVersion",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.btcstaking.v1.Query",
                "ParamsByVersion",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn params_by_btc_height(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryParamsByBtcHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryParamsByBtcHeightResponse>,
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
                "/babylon.btcstaking.v1.Query/ParamsByBTCHeight",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.btcstaking.v1.Query",
                "ParamsByBTCHeight",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn finality_providers(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryFinalityProvidersRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryFinalityProvidersResponse>,
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
                "/babylon.btcstaking.v1.Query/FinalityProviders",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.btcstaking.v1.Query",
                "FinalityProviders",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn finality_provider(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryFinalityProviderRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryFinalityProviderResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/babylon.btcstaking.v1.Query/FinalityProvider",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.btcstaking.v1.Query",
                "FinalityProvider",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn btc_delegations(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryBtcDelegationsRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryBtcDelegationsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/babylon.btcstaking.v1.Query/BTCDelegations");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.btcstaking.v1.Query",
                "BTCDelegations",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn finality_provider_delegations(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryFinalityProviderDelegationsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryFinalityProviderDelegationsResponse>,
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
                "/babylon.btcstaking.v1.Query/FinalityProviderDelegations",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.btcstaking.v1.Query",
                "FinalityProviderDelegations",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn btc_delegation(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryBtcDelegationRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryBtcDelegationResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/babylon.btcstaking.v1.Query/BTCDelegation");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.btcstaking.v1.Query",
                "BTCDelegation",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn largest_btc_re_org(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryLargestBtcReOrgRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryLargestBtcReOrgResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/babylon.btcstaking.v1.Query/LargestBtcReOrg",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.btcstaking.v1.Query",
                "LargestBtcReOrg",
            ));
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
        pub async fn create_finality_provider(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgCreateFinalityProvider>,
        ) -> std::result::Result<
            tonic::Response<super::MsgCreateFinalityProviderResponse>,
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
                "/babylon.btcstaking.v1.Msg/CreateFinalityProvider",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.btcstaking.v1.Msg",
                "CreateFinalityProvider",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn edit_finality_provider(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgEditFinalityProvider>,
        ) -> std::result::Result<
            tonic::Response<super::MsgEditFinalityProviderResponse>,
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
                "/babylon.btcstaking.v1.Msg/EditFinalityProvider",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.btcstaking.v1.Msg",
                "EditFinalityProvider",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn create_btc_delegation(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgCreateBtcDelegation>,
        ) -> std::result::Result<
            tonic::Response<super::MsgCreateBtcDelegationResponse>,
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
                "/babylon.btcstaking.v1.Msg/CreateBTCDelegation",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.btcstaking.v1.Msg",
                "CreateBTCDelegation",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn add_btc_delegation_inclusion_proof(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgAddBtcDelegationInclusionProof>,
        ) -> std::result::Result<
            tonic::Response<super::MsgAddBtcDelegationInclusionProofResponse>,
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
                "/babylon.btcstaking.v1.Msg/AddBTCDelegationInclusionProof",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.btcstaking.v1.Msg",
                "AddBTCDelegationInclusionProof",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn add_covenant_sigs(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgAddCovenantSigs>,
        ) -> std::result::Result<tonic::Response<super::MsgAddCovenantSigsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/babylon.btcstaking.v1.Msg/AddCovenantSigs");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.btcstaking.v1.Msg",
                "AddCovenantSigs",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn btc_undelegate(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgBtcUndelegate>,
        ) -> std::result::Result<tonic::Response<super::MsgBtcUndelegateResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/babylon.btcstaking.v1.Msg/BTCUndelegate");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.btcstaking.v1.Msg",
                "BTCUndelegate",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn selective_slashing_evidence(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgSelectiveSlashingEvidence>,
        ) -> std::result::Result<
            tonic::Response<super::MsgSelectiveSlashingEvidenceResponse>,
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
                "/babylon.btcstaking.v1.Msg/SelectiveSlashingEvidence",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "babylon.btcstaking.v1.Msg",
                "SelectiveSlashingEvidence",
            ));
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
                http::uri::PathAndQuery::from_static("/babylon.btcstaking.v1.Msg/UpdateParams");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("babylon.btcstaking.v1.Msg", "UpdateParams"));
            self.inner.unary(req, path, codec).await
        }
    }
}
