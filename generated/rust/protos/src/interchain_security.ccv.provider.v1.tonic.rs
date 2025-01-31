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
        pub async fn query_consumer_genesis(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryConsumerGenesisRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryConsumerGenesisResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/interchain_security.ccv.provider.v1.Query/QueryConsumerGenesis",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Query",
                "QueryConsumerGenesis",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn query_consumer_chains(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryConsumerChainsRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryConsumerChainsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/interchain_security.ccv.provider.v1.Query/QueryConsumerChains",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Query",
                "QueryConsumerChains",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn query_validator_consumer_addr(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryValidatorConsumerAddrRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryValidatorConsumerAddrResponse>,
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
                "/interchain_security.ccv.provider.v1.Query/QueryValidatorConsumerAddr",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Query",
                "QueryValidatorConsumerAddr",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn query_validator_provider_addr(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryValidatorProviderAddrRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryValidatorProviderAddrResponse>,
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
                "/interchain_security.ccv.provider.v1.Query/QueryValidatorProviderAddr",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Query",
                "QueryValidatorProviderAddr",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn query_throttle_state(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryThrottleStateRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryThrottleStateResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/interchain_security.ccv.provider.v1.Query/QueryThrottleState",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Query",
                "QueryThrottleState",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn query_registered_consumer_reward_denoms(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryRegisteredConsumerRewardDenomsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryRegisteredConsumerRewardDenomsResponse>,
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
                "/interchain_security.ccv.provider.v1.Query/QueryRegisteredConsumerRewardDenoms",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Query",
                "QueryRegisteredConsumerRewardDenoms",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn query_all_pairs_val_cons_addr_by_consumer(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryAllPairsValConsAddrByConsumerRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryAllPairsValConsAddrByConsumerResponse>,
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
                "/interchain_security.ccv.provider.v1.Query/QueryAllPairsValConsAddrByConsumer",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Query",
                "QueryAllPairsValConsAddrByConsumer",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn query_params(
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
            let path = http::uri::PathAndQuery::from_static(
                "/interchain_security.ccv.provider.v1.Query/QueryParams",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Query",
                "QueryParams",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn query_consumer_chain_opted_in_validators(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryConsumerChainOptedInValidatorsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryConsumerChainOptedInValidatorsResponse>,
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
                "/interchain_security.ccv.provider.v1.Query/QueryConsumerChainOptedInValidators",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Query",
                "QueryConsumerChainOptedInValidators",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn query_consumer_chains_validator_has_to_validate(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryConsumerChainsValidatorHasToValidateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryConsumerChainsValidatorHasToValidateResponse>,
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
                "/interchain_security.ccv.provider.v1.Query/QueryConsumerChainsValidatorHasToValidate",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Query",
                "QueryConsumerChainsValidatorHasToValidate",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn query_validator_consumer_commission_rate(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryValidatorConsumerCommissionRateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryValidatorConsumerCommissionRateResponse>,
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
                "/interchain_security.ccv.provider.v1.Query/QueryValidatorConsumerCommissionRate",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Query",
                "QueryValidatorConsumerCommissionRate",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn query_consumer_validators(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryConsumerValidatorsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryConsumerValidatorsResponse>,
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
                "/interchain_security.ccv.provider.v1.Query/QueryConsumerValidators",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Query",
                "QueryConsumerValidators",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn query_blocks_until_next_epoch(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryBlocksUntilNextEpochRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryBlocksUntilNextEpochResponse>,
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
                "/interchain_security.ccv.provider.v1.Query/QueryBlocksUntilNextEpoch",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Query",
                "QueryBlocksUntilNextEpoch",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn query_consumer_id_from_client_id(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryConsumerIdFromClientIdRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryConsumerIdFromClientIdResponse>,
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
                "/interchain_security.ccv.provider.v1.Query/QueryConsumerIdFromClientId",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Query",
                "QueryConsumerIdFromClientId",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn query_consumer_chain(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryConsumerChainRequest>,
        ) -> std::result::Result<tonic::Response<super::QueryConsumerChainResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/interchain_security.ccv.provider.v1.Query/QueryConsumerChain",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Query",
                "QueryConsumerChain",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn query_consumer_genesis_time(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryConsumerGenesisTimeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryConsumerGenesisTimeResponse>,
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
                "/interchain_security.ccv.provider.v1.Query/QueryConsumerGenesisTime",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Query",
                "QueryConsumerGenesisTime",
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
        pub async fn assign_consumer_key(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgAssignConsumerKey>,
        ) -> std::result::Result<tonic::Response<super::MsgAssignConsumerKeyResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/interchain_security.ccv.provider.v1.Msg/AssignConsumerKey",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Msg",
                "AssignConsumerKey",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn submit_consumer_misbehaviour(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgSubmitConsumerMisbehaviour>,
        ) -> std::result::Result<
            tonic::Response<super::MsgSubmitConsumerMisbehaviourResponse>,
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
                "/interchain_security.ccv.provider.v1.Msg/SubmitConsumerMisbehaviour",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Msg",
                "SubmitConsumerMisbehaviour",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn submit_consumer_double_voting(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgSubmitConsumerDoubleVoting>,
        ) -> std::result::Result<
            tonic::Response<super::MsgSubmitConsumerDoubleVotingResponse>,
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
                "/interchain_security.ccv.provider.v1.Msg/SubmitConsumerDoubleVoting",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Msg",
                "SubmitConsumerDoubleVoting",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn create_consumer(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgCreateConsumer>,
        ) -> std::result::Result<tonic::Response<super::MsgCreateConsumerResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/interchain_security.ccv.provider.v1.Msg/CreateConsumer",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Msg",
                "CreateConsumer",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_consumer(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgUpdateConsumer>,
        ) -> std::result::Result<tonic::Response<super::MsgUpdateConsumerResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/interchain_security.ccv.provider.v1.Msg/UpdateConsumer",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Msg",
                "UpdateConsumer",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn remove_consumer(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgRemoveConsumer>,
        ) -> std::result::Result<tonic::Response<super::MsgRemoveConsumerResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/interchain_security.ccv.provider.v1.Msg/RemoveConsumer",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Msg",
                "RemoveConsumer",
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
            let path = http::uri::PathAndQuery::from_static(
                "/interchain_security.ccv.provider.v1.Msg/UpdateParams",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Msg",
                "UpdateParams",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn opt_in(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgOptIn>,
        ) -> std::result::Result<tonic::Response<super::MsgOptInResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/interchain_security.ccv.provider.v1.Msg/OptIn",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Msg",
                "OptIn",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn opt_out(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgOptOut>,
        ) -> std::result::Result<tonic::Response<super::MsgOptOutResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/interchain_security.ccv.provider.v1.Msg/OptOut",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Msg",
                "OptOut",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn set_consumer_commission_rate(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgSetConsumerCommissionRate>,
        ) -> std::result::Result<
            tonic::Response<super::MsgSetConsumerCommissionRateResponse>,
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
                "/interchain_security.ccv.provider.v1.Msg/SetConsumerCommissionRate",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Msg",
                "SetConsumerCommissionRate",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn change_reward_denoms(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgChangeRewardDenoms>,
        ) -> std::result::Result<tonic::Response<super::MsgChangeRewardDenomsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/interchain_security.ccv.provider.v1.Msg/ChangeRewardDenoms",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "interchain_security.ccv.provider.v1.Msg",
                "ChangeRewardDenoms",
            ));
            self.inner.unary(req, path, codec).await
        }
    }
}
