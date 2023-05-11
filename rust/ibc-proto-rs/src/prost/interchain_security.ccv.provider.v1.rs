/// ConsumerAdditionProposal is a governance proposal on the provider chain to spawn a new consumer chain.
/// If it passes, then all validators on the provider chain are expected to validate the consumer chain at spawn time
/// or get slashed. It is recommended that spawn time occurs after the proposal end time.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsumerAdditionProposal {
    /// the title of the proposal
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// the description of the proposal
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// the proposed chain-id of the new consumer chain, must be different from all other consumer chain ids of the executing
    /// provider chain.
    #[prost(string, tag = "3")]
    pub chain_id: ::prost::alloc::string::String,
    /// the proposed initial height of new consumer chain.
    /// For a completely new chain, this will be {0,1}. However, it may be different if this is a chain that is converting to a consumer chain.
    #[prost(message, optional, tag = "4")]
    pub initial_height: ::core::option::Option<
        super::super::super::super::ibc::core::client::v1::Height,
    >,
    /// The hash of the consumer chain genesis state without the consumer CCV module genesis params.
    /// It is used for off-chain confirmation of genesis.json validity by validators and other parties.
    #[prost(bytes = "vec", tag = "5")]
    pub genesis_hash: ::prost::alloc::vec::Vec<u8>,
    /// The hash of the consumer chain binary that should be run by validators on chain initialization.
    /// It is used for off-chain confirmation of binary validity by validators and other parties.
    #[prost(bytes = "vec", tag = "6")]
    pub binary_hash: ::prost::alloc::vec::Vec<u8>,
    /// spawn time is the time on the provider chain at which the consumer chain genesis is finalized and all validators
    /// will be responsible for starting their consumer chain validator node.
    #[prost(message, optional, tag = "7")]
    pub spawn_time: ::core::option::Option<
        super::super::super::super::google::protobuf::Timestamp,
    >,
    /// Unbonding period for the consumer,
    /// which should be smaller than that of the provider in general.
    #[prost(message, optional, tag = "8")]
    pub unbonding_period: ::core::option::Option<
        super::super::super::super::google::protobuf::Duration,
    >,
    /// Sent CCV related IBC packets will timeout after this duration
    #[prost(message, optional, tag = "9")]
    pub ccv_timeout_period: ::core::option::Option<
        super::super::super::super::google::protobuf::Duration,
    >,
    /// Sent transfer related IBC packets will timeout after this duration
    #[prost(message, optional, tag = "10")]
    pub transfer_timeout_period: ::core::option::Option<
        super::super::super::super::google::protobuf::Duration,
    >,
    /// The fraction of tokens allocated to the consumer redistribution address
    /// during distribution events. The fraction is a string representing a
    /// decimal number. For example "0.75" would represent 75%.
    #[prost(string, tag = "11")]
    pub consumer_redistribution_fraction: ::prost::alloc::string::String,
    /// BlocksPerDistributionTransmission is the number of blocks between ibc-token-transfers from the consumer chain to the provider chain.
    /// On sending transmission event, `consumer_redistribution_fraction` of the accumulated tokens are sent to the consumer redistribution address.
    #[prost(int64, tag = "12")]
    pub blocks_per_distribution_transmission: i64,
    /// The number of historical info entries to persist in store.
    /// This param is a part of the cosmos sdk staking module. In the case of
    /// a ccv enabled consumer chain, the ccv module acts as the staking module.
    #[prost(int64, tag = "13")]
    pub historical_entries: i64,
}
/// ConsumerRemovalProposal is a governance proposal on the provider chain to remove (and stop) a consumer chain.
/// If it passes, all the consumer chain's state is removed from the provider chain. The outstanding unbonding
/// operation funds are released.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsumerRemovalProposal {
    /// the title of the proposal
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// the description of the proposal
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// the chain-id of the consumer chain to be stopped
    #[prost(string, tag = "3")]
    pub chain_id: ::prost::alloc::string::String,
    /// the time on the provider chain at which all validators are responsible to stop their consumer chain validator node
    #[prost(message, optional, tag = "4")]
    pub stop_time: ::core::option::Option<
        super::super::super::super::google::protobuf::Timestamp,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EquivocationProposal {
    /// the title of the proposal
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// the description of the proposal
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// the list of equivocations that will be processed
    #[prost(message, repeated, tag = "3")]
    pub equivocations: ::prost::alloc::vec::Vec<
        super::super::super::super::cosmos::evidence::v1beta1::Equivocation,
    >,
}
/// A persisted queue entry indicating that a slash packet data instance needs to be handled.
/// This type belongs in the "global" queue, to coordinate slash packet handling times between consumers.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GlobalSlashEntry {
    /// Block time that slash packet was received by provider chain.
    /// This field is used for store key iteration ordering.
    #[prost(message, optional, tag = "1")]
    pub recv_time: ::core::option::Option<
        super::super::super::super::google::protobuf::Timestamp,
    >,
    /// The consumer that sent a slash packet.
    #[prost(string, tag = "2")]
    pub consumer_chain_id: ::prost::alloc::string::String,
    /// The IBC sequence number of the recv packet.
    /// This field is used in the store key to ensure uniqueness.
    #[prost(uint64, tag = "3")]
    pub ibc_seq_num: u64,
    /// The provider's consensus address of the validator being slashed.
    /// This field is used to obtain validator power in HandleThrottleQueues.
    ///
    /// This field is not used in the store key, but is persisted in value bytes, see QueueGlobalSlashEntry.
    #[prost(bytes = "vec", tag = "4")]
    pub provider_val_cons_addr: ::prost::alloc::vec::Vec<u8>,
}
/// Params defines the parameters for CCV Provider module
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    #[prost(message, optional, tag = "1")]
    pub template_client: ::core::option::Option<
        super::super::super::super::ibc::lightclients::tendermint::v1::ClientState,
    >,
    /// TrustingPeriodFraction is used to compute the consumer and provider IBC client's TrustingPeriod from the chain defined UnbondingPeriod
    #[prost(string, tag = "2")]
    pub trusting_period_fraction: ::prost::alloc::string::String,
    /// Sent IBC packets will timeout after this duration
    #[prost(message, optional, tag = "3")]
    pub ccv_timeout_period: ::core::option::Option<
        super::super::super::super::google::protobuf::Duration,
    >,
    /// The channel initialization (IBC channel opening handshake) will timeout after this duration
    #[prost(message, optional, tag = "4")]
    pub init_timeout_period: ::core::option::Option<
        super::super::super::super::google::protobuf::Duration,
    >,
    /// The VSC packets sent by the provider will timeout after this duration.
    /// Note that unlike ccv_timeout_period which is an IBC param,
    /// the vsc_timeout_period is a provider-side param that enables the provider
    /// to timeout VSC packets even when a consumer chain is not live.
    #[prost(message, optional, tag = "5")]
    pub vsc_timeout_period: ::core::option::Option<
        super::super::super::super::google::protobuf::Duration,
    >,
    /// The period for which the slash meter is replenished
    #[prost(message, optional, tag = "6")]
    pub slash_meter_replenish_period: ::core::option::Option<
        super::super::super::super::google::protobuf::Duration,
    >,
    /// The fraction of total voting power that is replenished to the slash meter every replenish period.
    /// This param also serves as a maximum fraction of total voting power that the slash meter can hold.
    #[prost(string, tag = "7")]
    pub slash_meter_replenish_fraction: ::prost::alloc::string::String,
    /// The maximum amount of throttled slash or vsc matured packets
    /// that can be queued for a single consumer before the provider chain halts.
    #[prost(int64, tag = "8")]
    pub max_throttled_packets: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HandshakeMetadata {
    #[prost(string, tag = "1")]
    pub provider_fee_pool_addr: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub version: ::prost::alloc::string::String,
}
/// SlashAcks contains addesses of consumer chain validators
/// successfully slashed on the provider chain
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SlashAcks {
    #[prost(string, repeated, tag = "1")]
    pub addresses: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// ConsumerAdditionProposals holds pending governance proposals on the provider chain to spawn a new chain.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsumerAdditionProposals {
    /// proposals waiting for spawn_time to pass
    #[prost(message, repeated, tag = "1")]
    pub pending: ::prost::alloc::vec::Vec<ConsumerAdditionProposal>,
}
/// ConsumerRemovalProposals holds pending governance proposals on the provider chain to remove (and stop) a consumer chain.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsumerRemovalProposals {
    /// proposals waiting for stop_time to pass
    #[prost(message, repeated, tag = "1")]
    pub pending: ::prost::alloc::vec::Vec<ConsumerRemovalProposal>,
}
/// AddressList contains a list of consensus addresses
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddressList {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub addresses: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChannelToChain {
    #[prost(string, tag = "1")]
    pub channel_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
}
/// VscUnbondingOps contains the IDs of unbonding operations that are waiting for
/// at least one VSCMaturedPacket with vscID from a consumer chain
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VscUnbondingOps {
    #[prost(uint64, tag = "1")]
    pub vsc_id: u64,
    #[prost(uint64, repeated, tag = "2")]
    pub unbonding_op_ids: ::prost::alloc::vec::Vec<u64>,
}
/// UnbondingOp contains the ids of consumer chains that need to unbond before
/// the unbonding operation with the given ID can unbond
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnbondingOp {
    #[prost(uint64, tag = "1")]
    pub id: u64,
    /// consumer chains that are still unbonding
    #[prost(string, repeated, tag = "2")]
    pub unbonding_consumer_chains: ::prost::alloc::vec::Vec<
        ::prost::alloc::string::String,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InitTimeoutTimestamp {
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub timestamp: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VscSendTimestamp {
    #[prost(uint64, tag = "1")]
    pub vsc_id: u64,
    #[prost(message, optional, tag = "2")]
    pub timestamp: ::core::option::Option<
        super::super::super::super::google::protobuf::Timestamp,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KeyAssignmentReplacement {
    #[prost(bytes = "vec", tag = "1")]
    pub provider_addr: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub prev_c_key: ::core::option::Option<::tendermint_proto::crypto::PublicKey>,
    #[prost(int64, tag = "3")]
    pub power: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsumerGenesisRequest {
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsumerGenesisResponse {
    #[prost(message, optional, tag = "1")]
    pub genesis_state: ::core::option::Option<super::super::consumer::v1::GenesisState>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsumerChainsRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsumerChainsResponse {
    #[prost(message, repeated, tag = "1")]
    pub chains: ::prost::alloc::vec::Vec<Chain>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsumerChainStartProposalsRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsumerChainStartProposalsResponse {
    #[prost(message, optional, tag = "1")]
    pub proposals: ::core::option::Option<ConsumerAdditionProposals>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsumerChainStopProposalsRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsumerChainStopProposalsResponse {
    #[prost(message, optional, tag = "1")]
    pub proposals: ::core::option::Option<ConsumerRemovalProposals>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Chain {
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub client_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorConsumerAddrRequest {
    /// The id of the consumer chain
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// The consensus address of the validator on the provider chain
    #[prost(string, tag = "2")]
    pub provider_address: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorConsumerAddrResponse {
    /// The address of the validator on the consumer chain
    #[prost(string, tag = "1")]
    pub consumer_address: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorProviderAddrRequest {
    /// The id of the provider chain
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// The consensus address of the validator on the consumer chain
    #[prost(string, tag = "2")]
    pub consumer_address: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorProviderAddrResponse {
    /// The address of the validator on the provider chain
    #[prost(string, tag = "1")]
    pub provider_address: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryThrottleStateRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryThrottleStateResponse {
    /// current slash_meter state
    #[prost(int64, tag = "1")]
    pub slash_meter: i64,
    /// allowance of voting power units (int) that the slash meter is given per replenish period
    /// this also serves as the max value for the meter.
    #[prost(int64, tag = "2")]
    pub slash_meter_allowance: i64,
    /// next time the slash meter could potentially be replenished, iff it's not full
    #[prost(message, optional, tag = "3")]
    pub next_replenish_candidate: ::core::option::Option<
        super::super::super::super::google::protobuf::Timestamp,
    >,
    /// data relevant to currently throttled slash packets
    #[prost(message, repeated, tag = "4")]
    pub packets: ::prost::alloc::vec::Vec<ThrottledSlashPacket>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryThrottledConsumerPacketDataRequest {
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryThrottledConsumerPacketDataResponse {
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub size: u64,
    #[prost(message, repeated, tag = "3")]
    pub packet_data_instances: ::prost::alloc::vec::Vec<ThrottledPacketDataWrapper>,
}
/// A query wrapper type for the global entry and data relevant to a throttled slash packet.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ThrottledSlashPacket {
    #[prost(message, optional, tag = "1")]
    pub global_entry: ::core::option::Option<GlobalSlashEntry>,
    #[prost(message, optional, tag = "2")]
    pub data: ::core::option::Option<super::super::v1::SlashPacketData>,
}
/// ThrottledPacketDataWrapper contains either SlashPacketData or VSCMaturedPacketData
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ThrottledPacketDataWrapper {
    #[prost(oneof = "throttled_packet_data_wrapper::Data", tags = "1, 2")]
    pub data: ::core::option::Option<throttled_packet_data_wrapper::Data>,
}
/// Nested message and enum types in `ThrottledPacketDataWrapper`.
pub mod throttled_packet_data_wrapper {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Data {
        #[prost(message, tag = "1")]
        SlashPacket(super::super::super::v1::SlashPacketData),
        #[prost(message, tag = "2")]
        VscMaturedPacket(super::super::super::v1::VscMaturedPacketData),
    }
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
        pub async fn query_consumer_genesis(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryConsumerGenesisRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryConsumerGenesisResponse>,
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
                "/interchain_security.ccv.provider.v1.Query/QueryConsumerGenesis",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "interchain_security.ccv.provider.v1.Query",
                        "QueryConsumerGenesis",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// ConsumerChains queries active consumer chains supported by the provider
        /// chain
        pub async fn query_consumer_chains(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryConsumerChainsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryConsumerChainsResponse>,
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
                "/interchain_security.ccv.provider.v1.Query/QueryConsumerChains",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "interchain_security.ccv.provider.v1.Query",
                        "QueryConsumerChains",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// QueryConsumerChainStarts queries consumer chain start proposals.
        pub async fn query_consumer_chain_starts(
            &mut self,
            request: impl tonic::IntoRequest<
                super::QueryConsumerChainStartProposalsRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<super::QueryConsumerChainStartProposalsResponse>,
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
                "/interchain_security.ccv.provider.v1.Query/QueryConsumerChainStarts",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "interchain_security.ccv.provider.v1.Query",
                        "QueryConsumerChainStarts",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// QueryConsumerChainStops queries consumer chain stop proposals.
        pub async fn query_consumer_chain_stops(
            &mut self,
            request: impl tonic::IntoRequest<
                super::QueryConsumerChainStopProposalsRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<super::QueryConsumerChainStopProposalsResponse>,
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
                "/interchain_security.ccv.provider.v1.Query/QueryConsumerChainStops",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "interchain_security.ccv.provider.v1.Query",
                        "QueryConsumerChainStops",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// QueryValidatorConsumerAddr queries the address
        /// assigned by a validator for a consumer chain.
        pub async fn query_validator_consumer_addr(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryValidatorConsumerAddrRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryValidatorConsumerAddrResponse>,
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
                "/interchain_security.ccv.provider.v1.Query/QueryValidatorConsumerAddr",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "interchain_security.ccv.provider.v1.Query",
                        "QueryValidatorConsumerAddr",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// QueryProviderAddr returns the provider chain validator
        /// given a consumer chain validator address
        pub async fn query_validator_provider_addr(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryValidatorProviderAddrRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryValidatorProviderAddrResponse>,
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
                "/interchain_security.ccv.provider.v1.Query/QueryValidatorProviderAddr",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "interchain_security.ccv.provider.v1.Query",
                        "QueryValidatorProviderAddr",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// QueryThrottleState returns the main on-chain state relevant to currently throttled slash packets
        pub async fn query_throttle_state(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryThrottleStateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryThrottleStateResponse>,
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
                "/interchain_security.ccv.provider.v1.Query/QueryThrottleState",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "interchain_security.ccv.provider.v1.Query",
                        "QueryThrottleState",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// QueryThrottledConsumerPacketData returns a list of pending packet data instances
        /// (slash packet and vsc matured) for a single consumer chain
        pub async fn query_throttled_consumer_packet_data(
            &mut self,
            request: impl tonic::IntoRequest<
                super::QueryThrottledConsumerPacketDataRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<super::QueryThrottledConsumerPacketDataResponse>,
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
                "/interchain_security.ccv.provider.v1.Query/QueryThrottledConsumerPacketData",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "interchain_security.ccv.provider.v1.Query",
                        "QueryThrottledConsumerPacketData",
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
        async fn query_consumer_genesis(
            &self,
            request: tonic::Request<super::QueryConsumerGenesisRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryConsumerGenesisResponse>,
            tonic::Status,
        >;
        /// ConsumerChains queries active consumer chains supported by the provider
        /// chain
        async fn query_consumer_chains(
            &self,
            request: tonic::Request<super::QueryConsumerChainsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryConsumerChainsResponse>,
            tonic::Status,
        >;
        /// QueryConsumerChainStarts queries consumer chain start proposals.
        async fn query_consumer_chain_starts(
            &self,
            request: tonic::Request<super::QueryConsumerChainStartProposalsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryConsumerChainStartProposalsResponse>,
            tonic::Status,
        >;
        /// QueryConsumerChainStops queries consumer chain stop proposals.
        async fn query_consumer_chain_stops(
            &self,
            request: tonic::Request<super::QueryConsumerChainStopProposalsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryConsumerChainStopProposalsResponse>,
            tonic::Status,
        >;
        /// QueryValidatorConsumerAddr queries the address
        /// assigned by a validator for a consumer chain.
        async fn query_validator_consumer_addr(
            &self,
            request: tonic::Request<super::QueryValidatorConsumerAddrRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryValidatorConsumerAddrResponse>,
            tonic::Status,
        >;
        /// QueryProviderAddr returns the provider chain validator
        /// given a consumer chain validator address
        async fn query_validator_provider_addr(
            &self,
            request: tonic::Request<super::QueryValidatorProviderAddrRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryValidatorProviderAddrResponse>,
            tonic::Status,
        >;
        /// QueryThrottleState returns the main on-chain state relevant to currently throttled slash packets
        async fn query_throttle_state(
            &self,
            request: tonic::Request<super::QueryThrottleStateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryThrottleStateResponse>,
            tonic::Status,
        >;
        /// QueryThrottledConsumerPacketData returns a list of pending packet data instances
        /// (slash packet and vsc matured) for a single consumer chain
        async fn query_throttled_consumer_packet_data(
            &self,
            request: tonic::Request<super::QueryThrottledConsumerPacketDataRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryThrottledConsumerPacketDataResponse>,
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
                "/interchain_security.ccv.provider.v1.Query/QueryConsumerGenesis" => {
                    #[allow(non_camel_case_types)]
                    struct QueryConsumerGenesisSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryConsumerGenesisRequest>
                    for QueryConsumerGenesisSvc<T> {
                        type Response = super::QueryConsumerGenesisResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryConsumerGenesisRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).query_consumer_genesis(request).await
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
                        let method = QueryConsumerGenesisSvc(inner);
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
                "/interchain_security.ccv.provider.v1.Query/QueryConsumerChains" => {
                    #[allow(non_camel_case_types)]
                    struct QueryConsumerChainsSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryConsumerChainsRequest>
                    for QueryConsumerChainsSvc<T> {
                        type Response = super::QueryConsumerChainsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryConsumerChainsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).query_consumer_chains(request).await
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
                        let method = QueryConsumerChainsSvc(inner);
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
                "/interchain_security.ccv.provider.v1.Query/QueryConsumerChainStarts" => {
                    #[allow(non_camel_case_types)]
                    struct QueryConsumerChainStartsSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<
                        super::QueryConsumerChainStartProposalsRequest,
                    > for QueryConsumerChainStartsSvc<T> {
                        type Response = super::QueryConsumerChainStartProposalsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::QueryConsumerChainStartProposalsRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).query_consumer_chain_starts(request).await
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
                        let method = QueryConsumerChainStartsSvc(inner);
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
                "/interchain_security.ccv.provider.v1.Query/QueryConsumerChainStops" => {
                    #[allow(non_camel_case_types)]
                    struct QueryConsumerChainStopsSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<
                        super::QueryConsumerChainStopProposalsRequest,
                    > for QueryConsumerChainStopsSvc<T> {
                        type Response = super::QueryConsumerChainStopProposalsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::QueryConsumerChainStopProposalsRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).query_consumer_chain_stops(request).await
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
                        let method = QueryConsumerChainStopsSvc(inner);
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
                "/interchain_security.ccv.provider.v1.Query/QueryValidatorConsumerAddr" => {
                    #[allow(non_camel_case_types)]
                    struct QueryValidatorConsumerAddrSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<
                        super::QueryValidatorConsumerAddrRequest,
                    > for QueryValidatorConsumerAddrSvc<T> {
                        type Response = super::QueryValidatorConsumerAddrResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::QueryValidatorConsumerAddrRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).query_validator_consumer_addr(request).await
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
                        let method = QueryValidatorConsumerAddrSvc(inner);
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
                "/interchain_security.ccv.provider.v1.Query/QueryValidatorProviderAddr" => {
                    #[allow(non_camel_case_types)]
                    struct QueryValidatorProviderAddrSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<
                        super::QueryValidatorProviderAddrRequest,
                    > for QueryValidatorProviderAddrSvc<T> {
                        type Response = super::QueryValidatorProviderAddrResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::QueryValidatorProviderAddrRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).query_validator_provider_addr(request).await
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
                        let method = QueryValidatorProviderAddrSvc(inner);
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
                "/interchain_security.ccv.provider.v1.Query/QueryThrottleState" => {
                    #[allow(non_camel_case_types)]
                    struct QueryThrottleStateSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryThrottleStateRequest>
                    for QueryThrottleStateSvc<T> {
                        type Response = super::QueryThrottleStateResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryThrottleStateRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).query_throttle_state(request).await
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
                        let method = QueryThrottleStateSvc(inner);
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
                "/interchain_security.ccv.provider.v1.Query/QueryThrottledConsumerPacketData" => {
                    #[allow(non_camel_case_types)]
                    struct QueryThrottledConsumerPacketDataSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<
                        super::QueryThrottledConsumerPacketDataRequest,
                    > for QueryThrottledConsumerPacketDataSvc<T> {
                        type Response = super::QueryThrottledConsumerPacketDataResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::QueryThrottledConsumerPacketDataRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).query_throttled_consumer_packet_data(request).await
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
                        let method = QueryThrottledConsumerPacketDataSvc(inner);
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
        const NAME: &'static str = "interchain_security.ccv.provider.v1.Query";
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAssignConsumerKey {
    /// The chain id of the consumer chain to assign a consensus public key to
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// The validator address on the provider
    #[prost(string, tag = "2")]
    pub provider_addr: ::prost::alloc::string::String,
    /// The consensus public key to use on the consumer
    #[prost(message, optional, tag = "3")]
    pub consumer_key: ::core::option::Option<
        super::super::super::super::google::protobuf::Any,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAssignConsumerKeyResponse {}
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod msg_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Msg defines the Msg service.
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
        pub async fn assign_consumer_key(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgAssignConsumerKey>,
        ) -> std::result::Result<
            tonic::Response<super::MsgAssignConsumerKeyResponse>,
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
                "/interchain_security.ccv.provider.v1.Msg/AssignConsumerKey",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "interchain_security.ccv.provider.v1.Msg",
                        "AssignConsumerKey",
                    ),
                );
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
        async fn assign_consumer_key(
            &self,
            request: tonic::Request<super::MsgAssignConsumerKey>,
        ) -> std::result::Result<
            tonic::Response<super::MsgAssignConsumerKeyResponse>,
            tonic::Status,
        >;
    }
    /// Msg defines the Msg service.
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
                "/interchain_security.ccv.provider.v1.Msg/AssignConsumerKey" => {
                    #[allow(non_camel_case_types)]
                    struct AssignConsumerKeySvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgAssignConsumerKey>
                    for AssignConsumerKeySvc<T> {
                        type Response = super::MsgAssignConsumerKeyResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgAssignConsumerKey>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).assign_consumer_key(request).await
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
                        let method = AssignConsumerKeySvc(inner);
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
        const NAME: &'static str = "interchain_security.ccv.provider.v1.Msg";
    }
}
/// GenesisState defines the CCV provider chain genesis state
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// strictly positive and set to 1 (DefaultValsetUpdateID) for a new chain
    #[prost(uint64, tag = "1")]
    pub valset_update_id: u64,
    /// empty for a new chain
    #[prost(message, repeated, tag = "2")]
    pub consumer_states: ::prost::alloc::vec::Vec<ConsumerState>,
    /// empty for a new chain
    #[prost(message, repeated, tag = "3")]
    pub unbonding_ops: ::prost::alloc::vec::Vec<UnbondingOp>,
    /// empty for a new chain
    #[prost(message, optional, tag = "4")]
    pub mature_unbonding_ops: ::core::option::Option<
        super::super::v1::MaturedUnbondingOps,
    >,
    /// empty for a new chain
    #[prost(message, repeated, tag = "5")]
    pub valset_update_id_to_height: ::prost::alloc::vec::Vec<ValsetUpdateIdToHeight>,
    /// empty for a new chain
    #[prost(message, repeated, tag = "6")]
    pub consumer_addition_proposals: ::prost::alloc::vec::Vec<ConsumerAdditionProposal>,
    /// empty for a new chain
    #[prost(message, repeated, tag = "7")]
    pub consumer_removal_proposals: ::prost::alloc::vec::Vec<ConsumerRemovalProposal>,
    #[prost(message, optional, tag = "8")]
    pub params: ::core::option::Option<Params>,
    /// empty for a new chain
    #[prost(message, repeated, tag = "9")]
    pub validator_consumer_pubkeys: ::prost::alloc::vec::Vec<ValidatorConsumerPubKey>,
    /// empty for a new chain
    #[prost(message, repeated, tag = "10")]
    pub validators_by_consumer_addr: ::prost::alloc::vec::Vec<ValidatorByConsumerAddr>,
    /// empty for a new chain
    #[prost(message, repeated, tag = "11")]
    pub consumer_addrs_to_prune: ::prost::alloc::vec::Vec<ConsumerAddrsToPrune>,
}
/// consumer chain
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsumerState {
    /// ChainID defines the chain ID for the consumer chain
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// ChannelID defines the IBC channel ID for the consumer chain
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    /// ClientID defines the IBC client ID for the consumer chain
    #[prost(string, tag = "3")]
    pub client_id: ::prost::alloc::string::String,
    /// InitalHeight defines the initial block height for the consumer chain
    #[prost(uint64, tag = "4")]
    pub initial_height: u64,
    /// ConsumerGenesis defines the initial consumer chain genesis states
    #[prost(message, optional, tag = "5")]
    pub consumer_genesis: ::core::option::Option<
        super::super::consumer::v1::GenesisState,
    >,
    /// PendingValsetChanges defines the pending validator set changes for the consumer chain
    #[prost(message, repeated, tag = "6")]
    pub pending_valset_changes: ::prost::alloc::vec::Vec<
        super::super::v1::ValidatorSetChangePacketData,
    >,
    #[prost(string, repeated, tag = "7")]
    pub slash_downtime_ack: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// UnbondingOpsIndex defines the unbonding operations waiting on this consumer chain
    #[prost(message, repeated, tag = "8")]
    pub unbonding_ops_index: ::prost::alloc::vec::Vec<VscUnbondingOps>,
}
/// ValsetUpdateIdToHeight defines the genesis information for the mapping
/// of each valset udpate id to a block height
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValsetUpdateIdToHeight {
    #[prost(uint64, tag = "1")]
    pub valset_update_id: u64,
    #[prost(uint64, tag = "2")]
    pub height: u64,
}
/// Used to serialize the ValidatorConsumerPubKey index from key assignment
/// ValidatorConsumerPubKey: (chainID, providerAddr consAddr) -> consumerKey tmprotocrypto.PublicKey
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorConsumerPubKey {
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "2")]
    pub provider_addr: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub consumer_key: ::core::option::Option<::tendermint_proto::crypto::PublicKey>,
}
/// Used to serialize the ValidatorConsumerAddr index from key assignment
/// ValidatorByConsumerAddr: (chainID, consumerAddr consAddr) -> providerAddr consAddr
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorByConsumerAddr {
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "2")]
    pub consumer_addr: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub provider_addr: ::prost::alloc::vec::Vec<u8>,
}
/// Used to serialize the ConsumerAddrsToPrune index from key assignment
/// ConsumerAddrsToPrune: (chainID, vscID uint64) -> consumerAddrs AddressList
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsumerAddrsToPrune {
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub vsc_id: u64,
    #[prost(message, optional, tag = "3")]
    pub consumer_addrs: ::core::option::Option<AddressList>,
}
