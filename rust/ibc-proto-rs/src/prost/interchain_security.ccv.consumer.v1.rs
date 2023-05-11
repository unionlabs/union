/// Params defines the parameters for CCV consumer module
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// TODO: Remove enabled flag and find a better way to setup e2e tests
    /// See: <https://github.com/cosmos/interchain-security/issues/339>
    #[prost(bool, tag = "1")]
    pub enabled: bool,
    /// /////////////////////
    /// Distribution Params
    /// Number of blocks between ibc-token-transfers from the consumer chain to
    /// the provider chain. Note that at this transmission event a fraction of
    /// the accumulated tokens are divided and sent consumer redistribution
    /// address.
    #[prost(int64, tag = "2")]
    pub blocks_per_distribution_transmission: i64,
    /// Channel, and provider-chain receiving address to send distribution token
    /// transfers over. These parameters is auto-set during the consumer <->
    /// provider handshake procedure.
    #[prost(string, tag = "3")]
    pub distribution_transmission_channel: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub provider_fee_pool_addr_str: ::prost::alloc::string::String,
    /// Sent CCV related IBC packets will timeout after this duration
    #[prost(message, optional, tag = "5")]
    pub ccv_timeout_period: ::core::option::Option<
        super::super::super::super::google::protobuf::Duration,
    >,
    /// Sent transfer related IBC packets will timeout after this duration
    #[prost(message, optional, tag = "6")]
    pub transfer_timeout_period: ::core::option::Option<
        super::super::super::super::google::protobuf::Duration,
    >,
    /// The fraction of tokens allocated to the consumer redistribution address
    /// during distribution events. The fraction is a string representing a
    /// decimal number. For example "0.75" would represent 75%.
    #[prost(string, tag = "7")]
    pub consumer_redistribution_fraction: ::prost::alloc::string::String,
    /// The number of historical info entries to persist in store.
    /// This param is a part of the cosmos sdk staking module. In the case of
    /// a ccv enabled consumer chain, the ccv module acts as the staking module.
    #[prost(int64, tag = "8")]
    pub historical_entries: i64,
    /// Unbonding period for the consumer,
    /// which should be smaller than that of the provider in general.
    #[prost(message, optional, tag = "9")]
    pub unbonding_period: ::core::option::Option<
        super::super::super::super::google::protobuf::Duration,
    >,
    /// The threshold for the percentage of validators at the bottom of the set who
    /// can opt out of running the consumer chain without being punished. For example, a
    /// value of 0.05 means that the validators in the bottom 5% of the set can opt out
    #[prost(string, tag = "10")]
    pub soft_opt_out_threshold: ::prost::alloc::string::String,
}
/// LastTransmissionBlockHeight is the last time validator holding
/// pools were transmitted to the provider chain
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LastTransmissionBlockHeight {
    #[prost(int64, tag = "1")]
    pub height: i64,
}
/// CrossChainValidator defines the validators for CCV consumer module
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CrossChainValidator {
    #[prost(bytes = "vec", tag = "1")]
    pub address: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag = "2")]
    pub power: i64,
    /// pubkey is the consensus public key of the validator, as a Protobuf Any.
    #[prost(message, optional, tag = "3")]
    pub pubkey: ::core::option::Option<
        super::super::super::super::google::protobuf::Any,
    >,
}
/// MaturingVSCPacket contains the maturing time of a received VSCPacket
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MaturingVscPacket {
    #[prost(uint64, tag = "1")]
    pub vsc_id: u64,
    #[prost(message, optional, tag = "2")]
    pub maturity_time: ::core::option::Option<
        super::super::super::super::google::protobuf::Timestamp,
    >,
}
/// GenesisState defines the CCV consumer chain genesis state
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
    /// empty for a new chain, filled in on restart.
    #[prost(string, tag = "2")]
    pub provider_client_id: ::prost::alloc::string::String,
    /// empty for a new chain, filled in on restart.
    #[prost(string, tag = "3")]
    pub provider_channel_id: ::prost::alloc::string::String,
    /// true for new chain GenesisState, false for chain restart.
    #[prost(bool, tag = "4")]
    pub new_chain: bool,
    /// ProviderClientState filled in on new chain, nil on restart.
    #[prost(message, optional, tag = "5")]
    pub provider_client_state: ::core::option::Option<
        super::super::super::super::ibc::lightclients::tendermint::v1::ClientState,
    >,
    /// ProviderConsensusState filled in on new chain, nil on restart.
    #[prost(message, optional, tag = "6")]
    pub provider_consensus_state: ::core::option::Option<
        super::super::super::super::ibc::lightclients::tendermint::v1::ConsensusState,
    >,
    /// MaturingPackets nil on new chain, filled in on restart.
    #[prost(message, repeated, tag = "7")]
    pub maturing_packets: ::prost::alloc::vec::Vec<MaturingVscPacket>,
    /// InitialValset filled in on new chain and on restart.
    #[prost(message, repeated, tag = "8")]
    pub initial_val_set: ::prost::alloc::vec::Vec<
        ::tendermint_proto::abci::ValidatorUpdate,
    >,
    /// HeightToValsetUpdateId nil on new chain, filled in on restart.
    #[prost(message, repeated, tag = "9")]
    pub height_to_valset_update_id: ::prost::alloc::vec::Vec<HeightToValsetUpdateId>,
    /// OutstandingDowntimes nil on new chain, filled  in on restart.
    #[prost(message, repeated, tag = "10")]
    pub outstanding_downtime_slashing: ::prost::alloc::vec::Vec<OutstandingDowntime>,
    /// PendingConsumerPackets nil on new chain, filled in on restart.
    #[prost(message, optional, tag = "11")]
    pub pending_consumer_packets: ::core::option::Option<
        super::super::v1::ConsumerPacketDataList,
    >,
    /// LastTransmissionBlockHeight nil on new chain, filled in on restart.
    #[prost(message, optional, tag = "12")]
    pub last_transmission_block_height: ::core::option::Option<
        LastTransmissionBlockHeight,
    >,
}
/// HeightValsetUpdateID defines the genesis information for the mapping
/// of each block height to a valset update id
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HeightToValsetUpdateId {
    #[prost(uint64, tag = "1")]
    pub height: u64,
    #[prost(uint64, tag = "2")]
    pub valset_update_id: u64,
}
/// OutstandingDowntime defines the genesis information for each validator
/// flagged with an outstanding downtime slashing.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OutstandingDowntime {
    #[prost(string, tag = "1")]
    pub validator_consensus_address: ::prost::alloc::string::String,
}
/// NextFeeDistributionEstimate holds information about next fee distribution
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NextFeeDistributionEstimate {
    /// current block height at the time of querying
    #[prost(int64, tag = "1")]
    pub current_height: i64,
    /// block height at which last distribution took place
    #[prost(int64, tag = "2")]
    pub last_height: i64,
    /// block height at which next distribution will take place
    #[prost(int64, tag = "3")]
    pub next_height: i64,
    /// ratio between consumer and provider fee distribution
    #[prost(string, tag = "4")]
    pub distribution_fraction: ::prost::alloc::string::String,
    /// total accruead fees at the time of querying
    #[prost(string, tag = "5")]
    pub total: ::prost::alloc::string::String,
    /// amount distibuted to provider chain
    #[prost(string, tag = "6")]
    pub to_provider: ::prost::alloc::string::String,
    /// amount distributed (kept) by consumer chain
    #[prost(string, tag = "7")]
    pub to_consumer: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryNextFeeDistributionEstimateRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryNextFeeDistributionEstimateResponse {
    #[prost(message, optional, tag = "1")]
    pub data: ::core::option::Option<NextFeeDistributionEstimate>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
/// QueryParamsResponse is response type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// params holds all the parameters of this module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod query_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
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
        /// ConsumerGenesis queries the genesis state needed to start a consumer chain
        /// whose proposal has been accepted
        pub async fn query_next_fee_distribution(
            &mut self,
            request: impl tonic::IntoRequest<
                super::QueryNextFeeDistributionEstimateRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<super::QueryNextFeeDistributionEstimateResponse>,
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
                "/interchain_security.ccv.consumer.v1.Query/QueryNextFeeDistribution",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "interchain_security.ccv.consumer.v1.Query",
                        "QueryNextFeeDistribution",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// QueryParams queries the ccv/consumer module parameters.
        pub async fn query_params(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryParamsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryParamsResponse>,
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
                "/interchain_security.ccv.consumer.v1.Query/QueryParams",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "interchain_security.ccv.consumer.v1.Query",
                        "QueryParams",
                    ),
                );
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
        /// ConsumerGenesis queries the genesis state needed to start a consumer chain
        /// whose proposal has been accepted
        async fn query_next_fee_distribution(
            &self,
            request: tonic::Request<super::QueryNextFeeDistributionEstimateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryNextFeeDistributionEstimateResponse>,
            tonic::Status,
        >;
        /// QueryParams queries the ccv/consumer module parameters.
        async fn query_params(
            &self,
            request: tonic::Request<super::QueryParamsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryParamsResponse>,
            tonic::Status,
        >;
    }
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
                "/interchain_security.ccv.consumer.v1.Query/QueryNextFeeDistribution" => {
                    #[allow(non_camel_case_types)]
                    struct QueryNextFeeDistributionSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<
                        super::QueryNextFeeDistributionEstimateRequest,
                    > for QueryNextFeeDistributionSvc<T> {
                        type Response = super::QueryNextFeeDistributionEstimateResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::QueryNextFeeDistributionEstimateRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).query_next_fee_distribution(request).await
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
                        let method = QueryNextFeeDistributionSvc(inner);
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
                "/interchain_security.ccv.consumer.v1.Query/QueryParams" => {
                    #[allow(non_camel_case_types)]
                    struct QueryParamsSvc<T: Query>(pub Arc<T>);
                    impl<T: Query> tonic::server::UnaryService<super::QueryParamsRequest>
                    for QueryParamsSvc<T> {
                        type Response = super::QueryParamsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryParamsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).query_params(request).await
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
                        let method = QueryParamsSvc(inner);
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
        const NAME: &'static str = "interchain_security.ccv.consumer.v1.Query";
    }
}
