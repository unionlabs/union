/// IdentifiedClientState defines a client state with an additional client
/// identifier field.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IdentifiedClientState {
    /// client identifier
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    /// client state
    #[prost(message, optional, tag = "2")]
    pub client_state: ::core::option::Option<
        super::super::super::super::google::protobuf::Any,
    >,
}
/// ConsensusStateWithHeight defines a consensus state with an additional height
/// field.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusStateWithHeight {
    /// consensus state height
    #[prost(message, optional, tag = "1")]
    pub height: ::core::option::Option<Height>,
    /// consensus state
    #[prost(message, optional, tag = "2")]
    pub consensus_state: ::core::option::Option<
        super::super::super::super::google::protobuf::Any,
    >,
}
/// ClientConsensusStates defines all the stored consensus states for a given
/// client.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientConsensusStates {
    /// client identifier
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    /// consensus states and their heights associated with the client
    #[prost(message, repeated, tag = "2")]
    pub consensus_states: ::prost::alloc::vec::Vec<ConsensusStateWithHeight>,
}
/// ClientUpdateProposal is a governance proposal. If it passes, the substitute
/// client's latest consensus state is copied over to the subject client. The proposal
/// handler may fail if the subject and the substitute do not match in client and
/// chain parameters (with exception to latest height, frozen height, and chain-id).
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientUpdateProposal {
    /// the title of the update proposal
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// the description of the proposal
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// the client identifier for the client to be updated if the proposal passes
    #[prost(string, tag = "3")]
    pub subject_client_id: ::prost::alloc::string::String,
    /// the substitute client identifier for the client standing in for the subject
    /// client
    #[prost(string, tag = "4")]
    pub substitute_client_id: ::prost::alloc::string::String,
}
/// UpgradeProposal is a gov Content type for initiating an IBC breaking
/// upgrade.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpgradeProposal {
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub plan: ::core::option::Option<
        super::super::super::super::cosmos::upgrade::v1beta1::Plan,
    >,
    /// An UpgradedClientState must be provided to perform an IBC breaking upgrade.
    /// This will make the chain commit to the correct upgraded (self) client state
    /// before the upgrade occurs, so that connecting chains can verify that the
    /// new upgraded client is valid by verifying a proof on the previous version
    /// of the chain. This will allow IBC connections to persist smoothly across
    /// planned chain upgrades
    #[prost(message, optional, tag = "4")]
    pub upgraded_client_state: ::core::option::Option<
        super::super::super::super::google::protobuf::Any,
    >,
}
/// Height is a monotonically increasing data type
/// that can be compared against another Height for the purposes of updating and
/// freezing clients
///
/// Normally the RevisionHeight is incremented at each height while keeping
/// RevisionNumber the same. However some consensus algorithms may choose to
/// reset the height in certain conditions e.g. hard forks, state-machine
/// breaking changes In these cases, the RevisionNumber is incremented so that
/// height continues to be monitonically increasing even as the RevisionHeight
/// gets reset
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "json-schema", derive(::schemars::JsonSchema))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Height {
    /// the revision that the client is currently on
    #[prost(uint64, tag = "1")]
    #[serde(default)]
    pub revision_number: u64,
    /// the height within the given revision
    #[prost(uint64, tag = "2")]
    #[serde(default)]
    pub revision_height: u64,
}
/// Params defines the set of IBC light client parameters.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// allowed_clients defines the list of allowed client state types which can be created
    /// and interacted with. If a client type is removed from the allowed clients list, usage
    /// of this client will be disabled until it is added again to the list.
    #[prost(string, repeated, tag = "1")]
    pub allowed_clients: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// GenesisState defines the ibc client submodule's genesis state.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// client states with their corresponding identifiers
    #[prost(message, repeated, tag = "1")]
    pub clients: ::prost::alloc::vec::Vec<IdentifiedClientState>,
    /// consensus states from each client
    #[prost(message, repeated, tag = "2")]
    pub clients_consensus: ::prost::alloc::vec::Vec<ClientConsensusStates>,
    /// metadata from each client
    #[prost(message, repeated, tag = "3")]
    pub clients_metadata: ::prost::alloc::vec::Vec<IdentifiedGenesisMetadata>,
    #[prost(message, optional, tag = "4")]
    pub params: ::core::option::Option<Params>,
    /// create localhost on initialization
    #[prost(bool, tag = "5")]
    pub create_localhost: bool,
    /// the sequence for the next generated client identifier
    #[prost(uint64, tag = "6")]
    pub next_client_sequence: u64,
}
/// GenesisMetadata defines the genesis type for metadata that clients may return
/// with ExportMetadata
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisMetadata {
    /// store key of metadata without clientID-prefix
    #[prost(bytes = "vec", tag = "1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    /// metadata value
    #[prost(bytes = "vec", tag = "2")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
/// IdentifiedGenesisMetadata has the client metadata with the corresponding
/// client id.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IdentifiedGenesisMetadata {
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub client_metadata: ::prost::alloc::vec::Vec<GenesisMetadata>,
}
/// QueryClientStateRequest is the request type for the Query/ClientState RPC
/// method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryClientStateRequest {
    /// client state unique identifier
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
}
/// QueryClientStateResponse is the response type for the Query/ClientState RPC
/// method. Besides the client state, it includes a proof and the height from
/// which the proof was retrieved.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryClientStateResponse {
    /// client state associated with the request identifier
    #[prost(message, optional, tag = "1")]
    pub client_state: ::core::option::Option<
        super::super::super::super::google::protobuf::Any,
    >,
    /// merkle proof of existence
    #[prost(bytes = "vec", tag = "2")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
    /// height at which the proof was retrieved
    #[prost(message, optional, tag = "3")]
    pub proof_height: ::core::option::Option<Height>,
}
/// QueryClientStatesRequest is the request type for the Query/ClientStates RPC
/// method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryClientStatesRequest {
    /// pagination request
    #[prost(message, optional, tag = "1")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageRequest,
    >,
}
/// QueryClientStatesResponse is the response type for the Query/ClientStates RPC
/// method.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryClientStatesResponse {
    /// list of stored ClientStates of the chain.
    #[prost(message, repeated, tag = "1")]
    pub client_states: ::prost::alloc::vec::Vec<IdentifiedClientState>,
    /// pagination response
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageResponse,
    >,
}
/// QueryConsensusStateRequest is the request type for the Query/ConsensusState
/// RPC method. Besides the consensus state, it includes a proof and the height
/// from which the proof was retrieved.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsensusStateRequest {
    /// client identifier
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    /// consensus state revision number
    #[prost(uint64, tag = "2")]
    pub revision_number: u64,
    /// consensus state revision height
    #[prost(uint64, tag = "3")]
    pub revision_height: u64,
    /// latest_height overrrides the height field and queries the latest stored
    /// ConsensusState
    #[prost(bool, tag = "4")]
    pub latest_height: bool,
}
/// QueryConsensusStateResponse is the response type for the Query/ConsensusState
/// RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsensusStateResponse {
    /// consensus state associated with the client identifier at the given height
    #[prost(message, optional, tag = "1")]
    pub consensus_state: ::core::option::Option<
        super::super::super::super::google::protobuf::Any,
    >,
    /// merkle proof of existence
    #[prost(bytes = "vec", tag = "2")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
    /// height at which the proof was retrieved
    #[prost(message, optional, tag = "3")]
    pub proof_height: ::core::option::Option<Height>,
}
/// QueryConsensusStatesRequest is the request type for the Query/ConsensusStates
/// RPC method.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsensusStatesRequest {
    /// client identifier
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    /// pagination request
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageRequest,
    >,
}
/// QueryConsensusStatesResponse is the response type for the
/// Query/ConsensusStates RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsensusStatesResponse {
    /// consensus states associated with the identifier
    #[prost(message, repeated, tag = "1")]
    pub consensus_states: ::prost::alloc::vec::Vec<ConsensusStateWithHeight>,
    /// pagination response
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageResponse,
    >,
}
/// QueryConsensusStateHeightsRequest is the request type for Query/ConsensusStateHeights
/// RPC method.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsensusStateHeightsRequest {
    /// client identifier
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    /// pagination request
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageRequest,
    >,
}
/// QueryConsensusStateHeightsResponse is the response type for the
/// Query/ConsensusStateHeights RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsensusStateHeightsResponse {
    /// consensus state heights
    #[prost(message, repeated, tag = "1")]
    pub consensus_state_heights: ::prost::alloc::vec::Vec<Height>,
    /// pagination response
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageResponse,
    >,
}
/// QueryClientStatusRequest is the request type for the Query/ClientStatus RPC
/// method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryClientStatusRequest {
    /// client unique identifier
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
}
/// QueryClientStatusResponse is the response type for the Query/ClientStatus RPC
/// method. It returns the current status of the IBC client.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryClientStatusResponse {
    #[prost(string, tag = "1")]
    pub status: ::prost::alloc::string::String,
}
/// QueryClientParamsRequest is the request type for the Query/ClientParams RPC
/// method.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryClientParamsRequest {}
/// QueryClientParamsResponse is the response type for the Query/ClientParams RPC
/// method.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryClientParamsResponse {
    /// params defines the parameters of the module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
/// QueryUpgradedClientStateRequest is the request type for the
/// Query/UpgradedClientState RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryUpgradedClientStateRequest {}
/// QueryUpgradedClientStateResponse is the response type for the
/// Query/UpgradedClientState RPC method.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryUpgradedClientStateResponse {
    /// client state associated with the request identifier
    #[prost(message, optional, tag = "1")]
    pub upgraded_client_state: ::core::option::Option<
        super::super::super::super::google::protobuf::Any,
    >,
}
/// QueryUpgradedConsensusStateRequest is the request type for the
/// Query/UpgradedConsensusState RPC method
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryUpgradedConsensusStateRequest {}
/// QueryUpgradedConsensusStateResponse is the response type for the
/// Query/UpgradedConsensusState RPC method.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryUpgradedConsensusStateResponse {
    /// Consensus state associated with the request identifier
    #[prost(message, optional, tag = "1")]
    pub upgraded_consensus_state: ::core::option::Option<
        super::super::super::super::google::protobuf::Any,
    >,
}
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod query_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Query provides defines the gRPC querier service
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
        /// ClientState queries an IBC light client.
        pub async fn client_state(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryClientStateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryClientStateResponse>,
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
                "/ibc.core.client.v1.Query/ClientState",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.client.v1.Query", "ClientState"));
            self.inner.unary(req, path, codec).await
        }
        /// ClientStates queries all the IBC light clients of a chain.
        pub async fn client_states(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryClientStatesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryClientStatesResponse>,
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
                "/ibc.core.client.v1.Query/ClientStates",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.client.v1.Query", "ClientStates"));
            self.inner.unary(req, path, codec).await
        }
        /// ConsensusState queries a consensus state associated with a client state at
        /// a given height.
        pub async fn consensus_state(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryConsensusStateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryConsensusStateResponse>,
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
                "/ibc.core.client.v1.Query/ConsensusState",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.client.v1.Query", "ConsensusState"));
            self.inner.unary(req, path, codec).await
        }
        /// ConsensusStates queries all the consensus state associated with a given
        /// client.
        pub async fn consensus_states(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryConsensusStatesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryConsensusStatesResponse>,
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
                "/ibc.core.client.v1.Query/ConsensusStates",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.client.v1.Query", "ConsensusStates"));
            self.inner.unary(req, path, codec).await
        }
        /// ConsensusStateHeights queries the height of every consensus states associated with a given client.
        pub async fn consensus_state_heights(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryConsensusStateHeightsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryConsensusStateHeightsResponse>,
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
                "/ibc.core.client.v1.Query/ConsensusStateHeights",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("ibc.core.client.v1.Query", "ConsensusStateHeights"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Status queries the status of an IBC client.
        pub async fn client_status(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryClientStatusRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryClientStatusResponse>,
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
                "/ibc.core.client.v1.Query/ClientStatus",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.client.v1.Query", "ClientStatus"));
            self.inner.unary(req, path, codec).await
        }
        /// ClientParams queries all parameters of the ibc client submodule.
        pub async fn client_params(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryClientParamsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryClientParamsResponse>,
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
                "/ibc.core.client.v1.Query/ClientParams",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.client.v1.Query", "ClientParams"));
            self.inner.unary(req, path, codec).await
        }
        /// UpgradedClientState queries an Upgraded IBC light client.
        pub async fn upgraded_client_state(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryUpgradedClientStateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryUpgradedClientStateResponse>,
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
                "/ibc.core.client.v1.Query/UpgradedClientState",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("ibc.core.client.v1.Query", "UpgradedClientState"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// UpgradedConsensusState queries an Upgraded IBC consensus state.
        pub async fn upgraded_consensus_state(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryUpgradedConsensusStateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryUpgradedConsensusStateResponse>,
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
                "/ibc.core.client.v1.Query/UpgradedConsensusState",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("ibc.core.client.v1.Query", "UpgradedConsensusState"),
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
        /// ClientState queries an IBC light client.
        async fn client_state(
            &self,
            request: tonic::Request<super::QueryClientStateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryClientStateResponse>,
            tonic::Status,
        >;
        /// ClientStates queries all the IBC light clients of a chain.
        async fn client_states(
            &self,
            request: tonic::Request<super::QueryClientStatesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryClientStatesResponse>,
            tonic::Status,
        >;
        /// ConsensusState queries a consensus state associated with a client state at
        /// a given height.
        async fn consensus_state(
            &self,
            request: tonic::Request<super::QueryConsensusStateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryConsensusStateResponse>,
            tonic::Status,
        >;
        /// ConsensusStates queries all the consensus state associated with a given
        /// client.
        async fn consensus_states(
            &self,
            request: tonic::Request<super::QueryConsensusStatesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryConsensusStatesResponse>,
            tonic::Status,
        >;
        /// ConsensusStateHeights queries the height of every consensus states associated with a given client.
        async fn consensus_state_heights(
            &self,
            request: tonic::Request<super::QueryConsensusStateHeightsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryConsensusStateHeightsResponse>,
            tonic::Status,
        >;
        /// Status queries the status of an IBC client.
        async fn client_status(
            &self,
            request: tonic::Request<super::QueryClientStatusRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryClientStatusResponse>,
            tonic::Status,
        >;
        /// ClientParams queries all parameters of the ibc client submodule.
        async fn client_params(
            &self,
            request: tonic::Request<super::QueryClientParamsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryClientParamsResponse>,
            tonic::Status,
        >;
        /// UpgradedClientState queries an Upgraded IBC light client.
        async fn upgraded_client_state(
            &self,
            request: tonic::Request<super::QueryUpgradedClientStateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryUpgradedClientStateResponse>,
            tonic::Status,
        >;
        /// UpgradedConsensusState queries an Upgraded IBC consensus state.
        async fn upgraded_consensus_state(
            &self,
            request: tonic::Request<super::QueryUpgradedConsensusStateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryUpgradedConsensusStateResponse>,
            tonic::Status,
        >;
    }
    /// Query provides defines the gRPC querier service
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
                "/ibc.core.client.v1.Query/ClientState" => {
                    #[allow(non_camel_case_types)]
                    struct ClientStateSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryClientStateRequest>
                    for ClientStateSvc<T> {
                        type Response = super::QueryClientStateResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryClientStateRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).client_state(request).await
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
                        let method = ClientStateSvc(inner);
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
                "/ibc.core.client.v1.Query/ClientStates" => {
                    #[allow(non_camel_case_types)]
                    struct ClientStatesSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryClientStatesRequest>
                    for ClientStatesSvc<T> {
                        type Response = super::QueryClientStatesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryClientStatesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).client_states(request).await
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
                        let method = ClientStatesSvc(inner);
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
                "/ibc.core.client.v1.Query/ConsensusState" => {
                    #[allow(non_camel_case_types)]
                    struct ConsensusStateSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryConsensusStateRequest>
                    for ConsensusStateSvc<T> {
                        type Response = super::QueryConsensusStateResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryConsensusStateRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).consensus_state(request).await
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
                        let method = ConsensusStateSvc(inner);
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
                "/ibc.core.client.v1.Query/ConsensusStates" => {
                    #[allow(non_camel_case_types)]
                    struct ConsensusStatesSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryConsensusStatesRequest>
                    for ConsensusStatesSvc<T> {
                        type Response = super::QueryConsensusStatesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryConsensusStatesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).consensus_states(request).await
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
                        let method = ConsensusStatesSvc(inner);
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
                "/ibc.core.client.v1.Query/ConsensusStateHeights" => {
                    #[allow(non_camel_case_types)]
                    struct ConsensusStateHeightsSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<
                        super::QueryConsensusStateHeightsRequest,
                    > for ConsensusStateHeightsSvc<T> {
                        type Response = super::QueryConsensusStateHeightsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::QueryConsensusStateHeightsRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).consensus_state_heights(request).await
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
                        let method = ConsensusStateHeightsSvc(inner);
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
                "/ibc.core.client.v1.Query/ClientStatus" => {
                    #[allow(non_camel_case_types)]
                    struct ClientStatusSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryClientStatusRequest>
                    for ClientStatusSvc<T> {
                        type Response = super::QueryClientStatusResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryClientStatusRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).client_status(request).await
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
                        let method = ClientStatusSvc(inner);
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
                "/ibc.core.client.v1.Query/ClientParams" => {
                    #[allow(non_camel_case_types)]
                    struct ClientParamsSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryClientParamsRequest>
                    for ClientParamsSvc<T> {
                        type Response = super::QueryClientParamsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryClientParamsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).client_params(request).await
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
                        let method = ClientParamsSvc(inner);
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
                "/ibc.core.client.v1.Query/UpgradedClientState" => {
                    #[allow(non_camel_case_types)]
                    struct UpgradedClientStateSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryUpgradedClientStateRequest>
                    for UpgradedClientStateSvc<T> {
                        type Response = super::QueryUpgradedClientStateResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::QueryUpgradedClientStateRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).upgraded_client_state(request).await
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
                        let method = UpgradedClientStateSvc(inner);
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
                "/ibc.core.client.v1.Query/UpgradedConsensusState" => {
                    #[allow(non_camel_case_types)]
                    struct UpgradedConsensusStateSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<
                        super::QueryUpgradedConsensusStateRequest,
                    > for UpgradedConsensusStateSvc<T> {
                        type Response = super::QueryUpgradedConsensusStateResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::QueryUpgradedConsensusStateRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).upgraded_consensus_state(request).await
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
                        let method = UpgradedConsensusStateSvc(inner);
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
        const NAME: &'static str = "ibc.core.client.v1.Query";
    }
}
/// MsgCreateClient defines a message to create an IBC client
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateClient {
    /// light client state
    #[prost(message, optional, tag = "1")]
    pub client_state: ::core::option::Option<
        super::super::super::super::google::protobuf::Any,
    >,
    /// consensus state associated with the client that corresponds to a given
    /// height.
    #[prost(message, optional, tag = "2")]
    pub consensus_state: ::core::option::Option<
        super::super::super::super::google::protobuf::Any,
    >,
    /// signer address
    #[prost(string, tag = "3")]
    pub signer: ::prost::alloc::string::String,
}
/// MsgCreateClientResponse defines the Msg/CreateClient response type.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateClientResponse {}
/// MsgUpdateClient defines an sdk.Msg to update a IBC client state using
/// the given client message.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateClient {
    /// client unique identifier
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    /// client message to update the light client
    #[prost(message, optional, tag = "2")]
    pub client_message: ::core::option::Option<
        super::super::super::super::google::protobuf::Any,
    >,
    /// signer address
    #[prost(string, tag = "3")]
    pub signer: ::prost::alloc::string::String,
}
/// MsgUpdateClientResponse defines the Msg/UpdateClient response type.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateClientResponse {}
/// MsgUpgradeClient defines an sdk.Msg to upgrade an IBC client to a new client
/// state
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpgradeClient {
    /// client unique identifier
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    /// upgraded client state
    #[prost(message, optional, tag = "2")]
    pub client_state: ::core::option::Option<
        super::super::super::super::google::protobuf::Any,
    >,
    /// upgraded consensus state, only contains enough information to serve as a
    /// basis of trust in update logic
    #[prost(message, optional, tag = "3")]
    pub consensus_state: ::core::option::Option<
        super::super::super::super::google::protobuf::Any,
    >,
    /// proof that old chain committed to new client
    #[prost(bytes = "vec", tag = "4")]
    pub proof_upgrade_client: ::prost::alloc::vec::Vec<u8>,
    /// proof that old chain committed to new consensus state
    #[prost(bytes = "vec", tag = "5")]
    pub proof_upgrade_consensus_state: ::prost::alloc::vec::Vec<u8>,
    /// signer address
    #[prost(string, tag = "6")]
    pub signer: ::prost::alloc::string::String,
}
/// MsgUpgradeClientResponse defines the Msg/UpgradeClient response type.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpgradeClientResponse {}
/// MsgSubmitMisbehaviour defines an sdk.Msg type that submits Evidence for
/// light client misbehaviour.
/// Warning: DEPRECATED
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSubmitMisbehaviour {
    /// client unique identifier
    #[deprecated]
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    /// misbehaviour used for freezing the light client
    #[deprecated]
    #[prost(message, optional, tag = "2")]
    pub misbehaviour: ::core::option::Option<
        super::super::super::super::google::protobuf::Any,
    >,
    /// signer address
    #[deprecated]
    #[prost(string, tag = "3")]
    pub signer: ::prost::alloc::string::String,
}
/// MsgSubmitMisbehaviourResponse defines the Msg/SubmitMisbehaviour response
/// type.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSubmitMisbehaviourResponse {}
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod msg_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Msg defines the ibc/client Msg service.
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
        /// CreateClient defines a rpc handler method for MsgCreateClient.
        pub async fn create_client(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgCreateClient>,
        ) -> std::result::Result<
            tonic::Response<super::MsgCreateClientResponse>,
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
                "/ibc.core.client.v1.Msg/CreateClient",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.client.v1.Msg", "CreateClient"));
            self.inner.unary(req, path, codec).await
        }
        /// UpdateClient defines a rpc handler method for MsgUpdateClient.
        pub async fn update_client(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgUpdateClient>,
        ) -> std::result::Result<
            tonic::Response<super::MsgUpdateClientResponse>,
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
                "/ibc.core.client.v1.Msg/UpdateClient",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.client.v1.Msg", "UpdateClient"));
            self.inner.unary(req, path, codec).await
        }
        /// UpgradeClient defines a rpc handler method for MsgUpgradeClient.
        pub async fn upgrade_client(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgUpgradeClient>,
        ) -> std::result::Result<
            tonic::Response<super::MsgUpgradeClientResponse>,
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
                "/ibc.core.client.v1.Msg/UpgradeClient",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.client.v1.Msg", "UpgradeClient"));
            self.inner.unary(req, path, codec).await
        }
        /// SubmitMisbehaviour defines a rpc handler method for MsgSubmitMisbehaviour.
        pub async fn submit_misbehaviour(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgSubmitMisbehaviour>,
        ) -> std::result::Result<
            tonic::Response<super::MsgSubmitMisbehaviourResponse>,
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
                "/ibc.core.client.v1.Msg/SubmitMisbehaviour",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.core.client.v1.Msg", "SubmitMisbehaviour"));
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
        /// CreateClient defines a rpc handler method for MsgCreateClient.
        async fn create_client(
            &self,
            request: tonic::Request<super::MsgCreateClient>,
        ) -> std::result::Result<
            tonic::Response<super::MsgCreateClientResponse>,
            tonic::Status,
        >;
        /// UpdateClient defines a rpc handler method for MsgUpdateClient.
        async fn update_client(
            &self,
            request: tonic::Request<super::MsgUpdateClient>,
        ) -> std::result::Result<
            tonic::Response<super::MsgUpdateClientResponse>,
            tonic::Status,
        >;
        /// UpgradeClient defines a rpc handler method for MsgUpgradeClient.
        async fn upgrade_client(
            &self,
            request: tonic::Request<super::MsgUpgradeClient>,
        ) -> std::result::Result<
            tonic::Response<super::MsgUpgradeClientResponse>,
            tonic::Status,
        >;
        /// SubmitMisbehaviour defines a rpc handler method for MsgSubmitMisbehaviour.
        async fn submit_misbehaviour(
            &self,
            request: tonic::Request<super::MsgSubmitMisbehaviour>,
        ) -> std::result::Result<
            tonic::Response<super::MsgSubmitMisbehaviourResponse>,
            tonic::Status,
        >;
    }
    /// Msg defines the ibc/client Msg service.
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
                "/ibc.core.client.v1.Msg/CreateClient" => {
                    #[allow(non_camel_case_types)]
                    struct CreateClientSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgCreateClient>
                    for CreateClientSvc<T> {
                        type Response = super::MsgCreateClientResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgCreateClient>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).create_client(request).await
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
                        let method = CreateClientSvc(inner);
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
                "/ibc.core.client.v1.Msg/UpdateClient" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateClientSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgUpdateClient>
                    for UpdateClientSvc<T> {
                        type Response = super::MsgUpdateClientResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgUpdateClient>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).update_client(request).await
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
                        let method = UpdateClientSvc(inner);
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
                "/ibc.core.client.v1.Msg/UpgradeClient" => {
                    #[allow(non_camel_case_types)]
                    struct UpgradeClientSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgUpgradeClient>
                    for UpgradeClientSvc<T> {
                        type Response = super::MsgUpgradeClientResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgUpgradeClient>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).upgrade_client(request).await
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
                        let method = UpgradeClientSvc(inner);
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
                "/ibc.core.client.v1.Msg/SubmitMisbehaviour" => {
                    #[allow(non_camel_case_types)]
                    struct SubmitMisbehaviourSvc<T: Msg>(pub Arc<T>);
                    impl<
                        T: Msg,
                    > tonic::server::UnaryService<super::MsgSubmitMisbehaviour>
                    for SubmitMisbehaviourSvc<T> {
                        type Response = super::MsgSubmitMisbehaviourResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgSubmitMisbehaviour>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).submit_misbehaviour(request).await
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
                        let method = SubmitMisbehaviourSvc(inner);
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
        const NAME: &'static str = "ibc.core.client.v1.Msg";
    }
}
