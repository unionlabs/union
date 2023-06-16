use futures::Future;

use self::msgs::StateProof;

pub mod cosmos;
pub mod evm;

pub trait LightClient {
    // type SourceChain;

    /// The client state type that this light client stores about the counterparty.
    type ClientState;

    /// The consensus state type that this light client stores about the counterparty.
    type ConsensusState;

    type UpdateClientMessage;

    fn chain_id(&self) -> impl Future<Output = String> + '_;

    fn create_client(
        &self,
        _: Self::ClientState,
        _: Self::ConsensusState,
    ) -> impl Future<Output = String> + '_;

    fn update_client(
        &self,
        client_id: String,
        _: Self::UpdateClientMessage,
    ) -> impl Future<Output = ()> + '_;

    fn consensus_state_proof(
        &self,
        client_id: String,
        counterparty_height: msgs::Height,
        self_height: msgs::Height,
    ) -> impl Future<Output = StateProof<Self::ConsensusState>> + '_;

    fn client_state_proof(
        &self,
        client_id: String,
        self_height: msgs::Height,
    ) -> impl Future<Output = StateProof<Self::ClientState>> + '_;

    fn connection_state_proof(
        &self,
        connection_id: String,
        self_height: msgs::Height,
    ) -> impl Future<Output = StateProof<msgs::ConnectionEnd>> + '_;

    fn channel_state_proof(
        &self,
        channel_id: String,
        port_id: String,
        self_height: msgs::Height,
    ) -> impl Future<Output = StateProof<msgs::channel::Channel>> + '_;

    fn query_latest_height(&self) -> impl Future<Output = msgs::Height> + '_;

    fn query_client_state(&self, client_id: String)
        -> impl Future<Output = Self::ClientState> + '_;
}

#[derive(Debug)]
pub enum QueryHeight {
    Latest,
    Specific(msgs::Height),
}

pub trait Connect<C>: LightClient
where
    C: LightClient,
{
    // fn generate_counterparty_handshake_client_state(
    //     &self,
    //     self_state: C::ClientState,
    // ) -> impl Future<Output = C::ClientState> + '_;

    // CONNECTION HANDSHAKE

    fn connection_open_init(
        &self,
        _: msgs::connection::MsgConnectionOpenInit,
    ) -> impl Future<Output = String> + '_;

    fn connection_open_try(
        &self,
        _: msgs::connection::MsgConnectionOpenTry<C::ClientState>,
    ) -> impl Future<Output = String> + '_;

    fn connection_open_ack(
        &self,
        _: msgs::connection::MsgConnectionOpenAck<C::ClientState>,
    ) -> impl Future<Output = ()> + '_;

    fn connection_open_confirm(
        &self,
        _: msgs::connection::MsgConnectionOpenConfirm,
    ) -> impl Future<Output = ()> + '_;

    // CHANNEL HANDSHAKE

    fn channel_open_init(
        &self,
        _: msgs::channel::MsgChannelOpenInit,
    ) -> impl Future<Output = String> + '_;

    fn channel_open_try(
        &self,
        _: msgs::channel::MsgChannelOpenTry,
    ) -> impl Future<Output = String> + '_;

    fn channel_open_ack(
        &self,
        _: msgs::channel::MsgChannelOpenAck,
    ) -> impl Future<Output = ()> + '_;

    fn channel_open_confirm(
        &self,
        _: msgs::channel::MsgChannelOpenConfirm,
    ) -> impl Future<Output = ()> + '_;

    // PACKETS

    fn recv_packet(&self, _: msgs::channel::MsgRecvPacket) -> impl Future<Output = ()> + '_;

    // OTHER STUFF

    /// Generates the latest client state for the counterparty chain.
    fn generate_counterparty_client_state(
        &self,
        height: msgs::Height,
    ) -> impl Future<Output = C::ClientState> + '_;

    /// Generates the latest consensus state for the counterparty chain.
    fn generate_counterparty_consensus_state(
        &self,
        height: msgs::Height,
    ) -> impl Future<Output = C::ConsensusState> + '_;

    fn update_counterparty_client<'a>(
        &'a self,
        counterparty: &'a C,
        counterparty_client_id: String,
        update_from: msgs::Height,
        update_to: msgs::Height,
    ) -> impl Future<Output = msgs::Height> + 'a;
}

trait ChainSource {
    fn chain_id(&self) -> impl Future<Output = String> + '_;

    fn query_latest_block(&self) -> impl Future<Output = msgs::Height>;
}

// pub trait NativeEncoding {}

// TODO(benluelo): Flatten this module
pub mod msgs {
    #[derive(Debug, Clone, Copy)]
    pub struct Height {
        pub revision_number: u64,
        pub revision_height: u64,
    }

    // impl Height {
    //     pub fn increment(self) -> Self {
    //         Self {
    //             revision_number: self.revision_number,
    //             revision_height: self.revision_height + 1,
    //         }
    //     }
    // }

    #[derive(Debug, Clone)]
    pub struct UnknownEnumVariant<T>(T);

    #[derive(Debug, Clone)]
    pub struct MerklePrefix {
        pub key_prefix: Vec<u8>,
    }

    #[derive(Debug, Clone)]
    pub struct Fraction {
        pub numerator: u64,
        pub denominator: u64,
    }

    #[derive(Debug, Clone)]
    pub struct Duration {
        pub seconds: i64,
        pub nanos: i32,
    }

    #[derive(Debug, Clone)]
    pub struct Timestamp {
        pub seconds: i64,
        pub nanos: i32,
    }

    #[derive(Debug, Clone)]
    pub struct MerkleRoot {
        pub hash: Vec<u8>,
    }

    // pub mod ics23 {
    //     #[derive(Debug, Clone)]
    //     pub struct ProofSpec {
    //         pub leaf_spec: LeafOp,
    //         pub inner_spec: InnerSpec,
    //         // REVIEW(benluelo): Can these be < 0?
    //         pub max_depth: i32,
    //         pub min_depth: i32,
    //     }

    //     #[derive(Debug, Clone)]
    //     pub struct LeafOp {
    //         pub hash: HashOp,
    //         pub prehash_key: HashOp,
    //         pub prehash_value: HashOp,
    //         pub length: LengthOp,
    //         pub prefix: Vec<u8>,
    //     }

    //     #[derive(Debug, Clone)]
    //     pub struct InnerOp {
    //         pub hash: HashOp,
    //         pub prefix: Vec<u8>,
    //         pub suffix: Vec<u8>,
    //     }

    //     #[derive(Debug, Clone)]
    //     pub struct InnerSpec {
    //         pub child_order: Vec<i32>,
    //         pub child_size: i32,
    //         pub min_prefix_length: i32,
    //         pub max_prefix_length: i32,
    //         pub empty_child: Vec<u8>,
    //         pub hash: HashOp,
    //     }

    //     #[derive(Debug, Clone)]
    //     pub enum HashOp {
    //         NoHash = 0,
    //         Sha256 = 1,
    //         Sha512 = 2,
    //         Keccak = 3,
    //         Ripemd160 = 4,
    //         Bitcoin = 5,
    //         Sha512256 = 6,
    //     }

    //     #[derive(Debug, Clone)]
    //     pub enum LengthOp {
    //         NoPrefix = 0,
    //         VarProto = 1,
    //         VarRlp = 2,
    //         Fixed32Big = 3,
    //         Fixed32Little = 4,
    //         Fixed64Big = 5,
    //         Fixed64Little = 6,
    //         Require32Bytes = 7,
    //         Require64Bytes = 8,
    //     }

    //     impl ProofSpec {
    //         pub fn default_tendermint_proof_specs() -> [ProofSpec; 2] {
    //             [
    //                 ProofSpec {
    //                     leaf_spec: LeafOp {
    //                         hash: HashOp::Sha256,
    //                         prehash_key: HashOp::NoHash,
    //                         prehash_value: HashOp::Sha256,
    //                         length: LengthOp::VarProto,
    //                         prefix: [0].to_vec(),
    //                     },
    //                     inner_spec: InnerSpec {
    //                         child_order: vec![0, 1],
    //                         child_size: 33,
    //                         min_prefix_length: 4,
    //                         max_prefix_length: 12,
    //                         empty_child: vec![],
    //                         hash: HashOp::Sha256,
    //                     },
    //                     max_depth: 0,
    //                     min_depth: 0,
    //                 },
    //                 ProofSpec {
    //                     leaf_spec: LeafOp {
    //                         hash: HashOp::Sha256,
    //                         prehash_key: HashOp::NoHash,
    //                         prehash_value: HashOp::Sha256,
    //                         length: LengthOp::VarProto,
    //                         prefix: [0].to_vec(),
    //                     },
    //                     inner_spec: InnerSpec {
    //                         child_order: vec![0, 1],
    //                         child_size: 32,
    //                         min_prefix_length: 1,
    //                         max_prefix_length: 1,
    //                         empty_child: vec![],
    //                         hash: HashOp::Sha256,
    //                     },
    //                     max_depth: 0,
    //                     min_depth: 0,
    //                 },
    //             ]
    //         }
    //     }
    // }

    // pub mod tendermint {
    //     #[derive(Debug, Clone)]
    //     pub struct ClientState {
    //         pub chain_id: String,
    //         pub trust_level: super::Fraction,
    //         pub trusting_period: super::Duration,
    //         pub unbonding_period: super::Duration,
    //         pub max_clock_drift: super::Duration,
    //         pub frozen_height: super::Height,
    //         pub latest_height: super::Height,
    //         pub proof_specs: Vec<super::ics23::ProofSpec>,
    //         pub upgrade_path: Vec<String>,
    //     }
    // }

    pub mod cometbls {
        #[derive(Debug, Clone)]
        pub struct ClientState {
            pub chain_id: String,
            pub trust_level: super::Fraction,
            pub trusting_period: super::Duration,
            pub unbonding_period: super::Duration,
            pub max_clock_drift: super::Duration,
            pub frozen_height: super::Height,
        }

        #[derive(Debug, Clone)]
        pub struct ConsensusState {
            pub root: super::MerkleRoot,
            pub next_validators_hash: Vec<u8>,
        }
    }

    pub mod ethereum {
        // , IntoProto
        #[derive(Debug, Clone)]
        // #[proto(::protos::ibc::lightclients::ethereum::v1::ClientState)]
        pub struct ClientState {
            pub genesis_validators_root: Vec<u8>,
            pub min_sync_committee_participants: u64,
            pub genesis_time: u64,
            // #[proto(required, try_into)]
            pub fork_parameters: ForkParameters,
            pub seconds_per_slot: u64,
            pub slots_per_epoch: u64,
            pub epochs_per_sync_committee_period: u64,
            // #[proto(required, into)]
            pub trust_level: super::Fraction,
            pub trusting_period: u64,
            pub latest_slot: u64,
            // #[proto(required, into)]
            pub frozen_height: Option<super::Height>,
            pub counterparty_commitment_slot: u64,
        }

        // impl From<ethereum::ConsensusState> for ethereum_v1::ConsensusState {
        //     fn from(value: ethereum::ConsensusState) -> Self {
        //         Self {
        //             slot: value.slot,
        //             storage_root: value.storage_root,
        //             timestamp: value.timestamp,
        //             current_sync_committee: value.current_sync_committee,
        //             next_sync_committee: value.next_sync_committee,
        //         }
        //     }
        // }

        // impl TryFrom<ethereum_v1::ClientState> for ethereum::ClientState {
        //     type Error = MissingField;

        //     fn try_from(value: ethereum_v1::ClientState) -> Result<Self, Self::Error> {
        //         Ok(Self {
        //             genesis_validators_root: value.genesis_validators_root,
        //             min_sync_committee_participants: value.min_sync_committee_participants,
        //             genesis_time: value.genesis_time,
        //             fork_parameters: value
        //                 .fork_parameters
        //                 .ok_or(MissingField("fork_parameters"))?
        //                 .try_into()?,
        //             seconds_per_slot: value.seconds_per_slot,
        //             slots_per_epoch: value.slots_per_epoch,
        //             epochs_per_sync_committee_period: value.epochs_per_sync_committee_period,
        //             trust_level: value.trust_level.ok_or(MissingField("trust_level"))?.into(),
        //             trusting_period: value.trusting_period,
        //             latest_slot: value.latest_slot,
        //             frozen_height: value
        //                 .frozen_height
        //                 .ok_or(MissingField("frozen_height"))?
        //                 .into(),
        //             counterparty_commitment_slot: value.counterparty_commitment_slot,
        //         })
        //     }
        // }

        #[derive(Debug, Clone)]
        pub struct ForkParameters {
            // REVIEW(benluelo): Are these versions a fixed-length array? (in Fork as wel)
            pub genesis_fork_version: Vec<u8>,
            pub genesis_slot: u64,
            pub altair: Fork,
            pub bellatrix: Fork,
            pub capella: Fork,
            pub eip4844: Fork,
        }

        #[derive(Debug, Clone)]
        pub struct Fork {
            pub version: Vec<u8>,
            pub epoch: u64,
        }

        #[derive(Debug, Clone)]
        pub struct ConsensusState {
            pub slot: u64,
            pub storage_root: Vec<u8>,
            pub timestamp: u64,
            pub current_sync_committee: Vec<u8>,
            pub next_sync_committee: Vec<u8>,
        }

        #[derive(Debug)]
        pub struct Header {
            pub trusted_sync_committee: TrustedSyncCommittee,
            pub consensus_update: LightClientUpdate,
            pub account_update: AccountUpdate,
            pub timestamp: u64,
        }

        #[derive(Debug, Clone, PartialEq)]
        pub struct AccountUpdate {
            pub proofs: Vec<Proof>,
        }

        #[derive(Debug, Clone, PartialEq)]
        pub struct Proof {
            // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
            pub key: Vec<u8>,
            // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
            pub value: Vec<u8>,
            // #[serde(with = "::serde_utils::inner_base64")]
            pub proof: Vec<Vec<u8>>,
        }

        #[derive(Debug, Clone, PartialEq)]
        pub struct SyncAggregate {
            // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
            pub sync_committee_bits: Vec<u8>,
            // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
            pub sync_committee_signature: Vec<u8>,
        }

        #[derive(Debug)]
        pub struct TrustedSyncCommittee {
            pub trusted_height: super::Height,
            pub sync_committee: SyncCommittee,
            pub is_next: bool,
        }

        #[derive(Debug, Default)]
        pub struct SyncCommittee {
            pub pubkeys: Vec<Vec<u8>>,
            pub aggregate_pubkey: Vec<u8>,
        }

        #[derive(Debug)]
        pub struct LightClientUpdate {
            pub attested_header: LightClientHeader,
            pub next_sync_committee: SyncCommittee,
            // TODO(benluelo): vec<bytes32>
            pub next_sync_committee_branch: Vec<Vec<u8>>,
            pub finalized_header: LightClientHeader,
            pub finality_branch: Vec<Vec<u8>>,
            pub sync_aggregate: SyncAggregate,
            pub signature_slot: u64,
        }

        #[derive(Debug, Clone, PartialEq)]
        pub struct LightClientHeader {
            pub beacon: BeaconBlockHeader,
            pub execution: ExecutionPayloadHeader,
            // #[serde(with = "::serde_utils::inner_base64")]
            pub execution_branch: Vec<Vec<u8>>,
        }

        #[derive(Debug, Clone, PartialEq)]
        pub struct ExecutionPayloadHeader {
            // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
            pub parent_hash: Vec<u8>,
            // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
            pub fee_recipient: Vec<u8>,
            // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
            pub state_root: Vec<u8>,
            // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
            pub receipts_root: Vec<u8>,
            // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
            pub logs_bloom: Vec<u8>,
            // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
            pub prev_randao: Vec<u8>,
            pub block_number: u64,
            pub gas_limit: u64,
            pub gas_used: u64,
            pub timestamp: u64,
            // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
            pub extra_data: Vec<u8>,
            /// TODO(aeryz): U256
            // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
            pub base_fee_per_gas: Vec<u8>,
            // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
            pub block_hash: Vec<u8>,
            // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
            pub transactions_root: Vec<u8>,
            // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
            pub withdrawals_root: Vec<u8>,
        }

        #[derive(Debug, Clone, PartialEq)]
        pub struct BeaconBlockHeader {
            pub slot: u64,
            pub proposer_index: u64,
            // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
            pub parent_root: Vec<u8>,
            // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
            pub state_root: Vec<u8>,
            // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
            pub body_root: Vec<u8>,
        }
    }

    pub mod wasm {
        #[derive(Debug, Clone)]
        pub struct ClientState<Data> {
            pub data: Data,
            pub code_id: Vec<u8>,
            pub latest_height: super::Height,
        }

        impl<Data> crate::chain::ClientState for ClientState<Data> {
            fn height(&self) -> super::Height {
                self.latest_height
            }
        }

        #[derive(Debug, Clone)]
        pub struct ConsensusState<Data> {
            pub data: Data,
            pub timestamp: u64,
        }

        #[derive(Debug, Clone)]
        pub struct Header<Data> {
            pub data: Data,
            pub height: super::Height,
        }
    }

    pub mod connection {
        use super::{channel::Order, UnknownEnumVariant};

        #[derive(Debug, Clone)]
        pub struct Counterparty {
            pub client_id: String,
            pub connection_id: String,
            pub prefix: super::MerklePrefix,
        }

        #[derive(Debug, Clone)]
        pub struct Version {
            // TODO(benluelo): "The identifier field specifies a unique version identifier. A value of "1" specifies IBC 1.0.0."
            pub identifier: String,
            pub features: Vec<Order>,
        }

        #[derive(Debug, Clone)]
        pub enum State {
            /// Default State
            UninitializedUnspecified = 0,
            /// A connection end has just started the opening handshake.
            Init = 1,
            /// A connection end has acknowledged the handshake step on the counterparty
            /// chain.
            Tryopen = 2,
            /// A connection end has completed the handshake.
            Open = 3,
        }

        impl TryFrom<i32> for State {
            type Error = UnknownEnumVariant<i32>;

            fn try_from(value: i32) -> Result<Self, Self::Error> {
                match value {
                    0 => Ok(State::UninitializedUnspecified),
                    1 => Ok(State::Init),
                    2 => Ok(State::Tryopen),
                    3 => Ok(State::Open),
                    state => Err(UnknownEnumVariant(state)),
                }
            }
        }

        impl TryFrom<u8> for State {
            type Error = UnknownEnumVariant<u8>;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    0 => Ok(State::UninitializedUnspecified),
                    1 => Ok(State::Init),
                    2 => Ok(State::Tryopen),
                    3 => Ok(State::Open),
                    state => Err(UnknownEnumVariant(state)),
                }
            }
        }

        pub struct MsgConnectionOpenInit {
            pub client_id: String,
            pub counterparty: Counterparty,
            pub version: Version,
            pub delay_period: u64,
        }

        pub struct MsgConnectionOpenInitResponse {
            pub connection_id: String,
            pub inclusion_height: super::Height,
        }

        #[derive(Debug)]
        pub struct MsgConnectionOpenTry<ClientState> {
            pub client_id: String,
            pub client_state: ClientState,
            pub counterparty: Counterparty,
            pub delay_period: u64,
            pub counterparty_versions: Vec<Version>,
            pub proof_height: super::Height,
            pub proof_init: Vec<u8>,
            pub proof_client: Vec<u8>,
            pub proof_consensus: Vec<u8>,
            pub consensus_height: super::Height,
        }

        pub struct MsgConnectionOpenAck<ClientState> {
            pub connection_id: String,
            pub counterparty_connection_id: String,
            pub version: Version,
            pub client_state: ClientState,
            pub proof_height: super::Height,
            pub proof_try: Vec<u8>,
            pub proof_client: Vec<u8>,
            pub proof_consensus: Vec<u8>,
            pub consensus_height: super::Height,
        }

        pub struct MsgConnectionOpenConfirm {
            pub connection_id: String,
            pub proof_ack: Vec<u8>,
            pub proof_height: super::Height,
        }
    }

    pub mod channel {
        use super::UnknownEnumVariant;

        #[derive(Debug, Clone)]
        pub struct Channel {
            pub state: State,
            pub ordering: Order,
            pub counterparty: Counterparty,
            pub connection_hops: Vec<String>,
            pub version: String,
        }

        #[derive(Debug, Clone)]
        pub enum State {
            UninitializedUnspecified,
            Init,
            Tryopen,
            Open,
            Closed,
        }

        impl TryFrom<i32> for State {
            type Error = UnknownEnumVariant<i32>;

            fn try_from(value: i32) -> Result<Self, Self::Error> {
                match value {
                    0 => Ok(State::UninitializedUnspecified),
                    1 => Ok(State::Init),
                    2 => Ok(State::Tryopen),
                    3 => Ok(State::Open),
                    3 => Ok(State::Closed),
                    state => Err(UnknownEnumVariant(state)),
                }
            }
        }

        impl TryFrom<u8> for State {
            type Error = UnknownEnumVariant<u8>;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    0 => Ok(State::UninitializedUnspecified),
                    1 => Ok(State::Init),
                    2 => Ok(State::Tryopen),
                    3 => Ok(State::Open),
                    4 => Ok(State::Closed),
                    state => Err(UnknownEnumVariant(state)),
                }
            }
        }

        #[derive(
            Debug, PartialEq, Eq, Hash, strum::EnumString, strum::IntoStaticStr, Clone, Copy,
        )]
        pub enum Order {
            #[strum(serialize = "ORDER_UNORDERED")]
            Unordered,
            #[strum(serialize = "ORDER_ORDERED")]
            Ordered,
        }

        impl TryFrom<u8> for Order {
            type Error = UnknownEnumVariant<u8>;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    1 => Ok(Order::Ordered),
                    2 => Ok(Order::Unordered),
                    state => Err(UnknownEnumVariant(state)),
                }
            }
        }

        #[derive(Debug, Clone)]
        pub struct Counterparty {
            pub port_id: String,
            pub channel_id: String,
        }

        #[derive(Debug, Clone)]
        pub struct MsgChannelOpenInit {
            pub port_id: String,
            pub channel: Channel,
        }

        #[derive(Debug, Clone)]
        pub struct MsgChannelOpenInitResponse {
            pub channel_id: String,
            pub version: String,
        }

        #[derive(Debug, Clone)]
        pub struct MsgChannelOpenTry {
            pub port_id: String,
            pub channel: Channel,
            pub counterparty_version: String,
            pub proof_init: Vec<u8>,
            pub proof_height: super::Height,
        }

        #[derive(Debug, Clone)]
        pub struct MsgChannelOpenTryResponse {
            pub version: String,
            pub channel_id: String,
        }

        #[derive(Debug, Clone)]
        pub struct MsgChannelOpenAck {
            pub port_id: String,
            pub channel_id: String,
            pub counterparty_channel_id: String,
            pub counterparty_version: String,
            pub proof_try: Vec<u8>,
            pub proof_height: super::Height,
        }

        #[derive(Debug, Clone)]
        pub struct MsgChannelOpenConfirm {
            pub port_id: String,
            pub channel_id: String,
            pub proof_ack: Vec<u8>,
            pub proof_height: super::Height,
        }

        #[derive(Debug, Clone)]
        pub struct MsgRecvPacket {
            pub packet: Packet,
            pub proof_commitment: Vec<u8>,
            pub proof_height: super::Height,
        }

        #[derive(Debug, Clone)]
        pub struct Packet {
            pub sequence: u64,
            pub source_port: String,
            pub source_channel: String,
            pub destination_port: String,
            pub destination_channel: String,
            pub data: Vec<u8>,
            pub timeout_height: super::Height,
            pub timeout_timestamp: u64,
        }
    }

    #[derive(Debug, Clone)]
    pub struct StateProof<State> {
        /// client state associated with the request identifier
        pub state: State,
        /// merkle proof of existence
        pub proof: Vec<u8>,
        /// height at which the proof was retrieved
        pub proof_height: Height,
    }

    #[derive(Debug, Clone)]
    pub struct ConnectionEnd {
        /// client associated with this connection.
        pub client_id: String,
        /// IBC version which can be utilised to determine encodings or protocols for
        /// channels or packets utilising this connection.
        pub versions: Vec<connection::Version>,
        /// current state of the connection end.
        pub state: connection::State,
        /// counterparty chain associated with this connection.
        pub counterparty: connection::Counterparty,
        /// delay period that must pass before a consensus state can be used for
        /// packet-verification NOTE: delay period logic is only implemented by some
        /// clients.
        pub delay_period: u64,
    }
}

pub trait ClientState {
    fn height(&self) -> msgs::Height;
}

impl<T> ClientState for cosmos::Any<T>
where
    T: ClientState,
{
    fn height(&self) -> msgs::Height {
        self.0.height()
    }
}
