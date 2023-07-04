use futures::Future;
use ibc_types::{
    core::{
        channel::{
            channel::Channel, msg_channel_open_ack::MsgChannelOpenAck,
            msg_channel_open_confirm::MsgChannelOpenConfirm,
            msg_channel_open_init::MsgChannelOpenInit, msg_channel_open_try::MsgChannelOpenTry,
            msg_recv_packet::MsgRecvPacket,
        },
        client::height::Height,
        connection::{
            connection_end::ConnectionEnd, msg_channel_open_ack::MsgConnectionOpenAck,
            msg_channel_open_confirm::MsgConnectionOpenConfirm,
            msg_channel_open_init::MsgConnectionOpenInit,
            msg_channel_open_try::MsgConnectionOpenTry,
        },
    },
    google::protobuf::any::Any,
    lightclients::{cometbls, ethereum, wasm},
};

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
        counterparty_height: Height,
        self_height: Height,
    ) -> impl Future<Output = StateProof<Self::ConsensusState>> + '_;

    fn client_state_proof(
        &self,
        client_id: String,
        self_height: Height,
    ) -> impl Future<Output = StateProof<Self::ClientState>> + '_;

    fn connection_state_proof(
        &self,
        connection_id: String,
        self_height: Height,
    ) -> impl Future<Output = StateProof<ConnectionEnd>> + '_;

    fn channel_state_proof(
        &self,
        channel_id: String,
        port_id: String,
        self_height: Height,
    ) -> impl Future<Output = StateProof<Channel>> + '_;

    fn query_latest_height(&self) -> impl Future<Output = Height> + '_;

    fn query_client_state(&self, client_id: String)
        -> impl Future<Output = Self::ClientState> + '_;

    fn process_height_for_counterparty(&self, height: Height) -> impl Future<Output = Height> + '_;
}

#[derive(Debug)]
pub enum QueryHeight {
    Latest,
    Specific(Height),
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

    fn connection_open_init(&self, _: MsgConnectionOpenInit) -> impl Future<Output = String> + '_;

    fn connection_open_try(
        &self,
        _: MsgConnectionOpenTry<C::ClientState>,
    ) -> impl Future<Output = String> + '_;

    fn connection_open_ack(
        &self,
        _: MsgConnectionOpenAck<C::ClientState>,
    ) -> impl Future<Output = ()> + '_;

    fn connection_open_confirm(&self, _: MsgConnectionOpenConfirm)
        -> impl Future<Output = ()> + '_;

    // CHANNEL HANDSHAKE

    fn channel_open_init(&self, _: MsgChannelOpenInit) -> impl Future<Output = String> + '_;

    fn channel_open_try(&self, _: MsgChannelOpenTry) -> impl Future<Output = String> + '_;

    fn channel_open_ack(&self, _: MsgChannelOpenAck) -> impl Future<Output = ()> + '_;

    fn channel_open_confirm(&self, _: MsgChannelOpenConfirm) -> impl Future<Output = ()> + '_;

    // PACKETS

    fn recv_packet(&self, _: MsgRecvPacket) -> impl Future<Output = ()> + '_;

    // OTHER STUFF

    /// Generates the latest client state for the counterparty chain.
    fn generate_counterparty_client_state(
        &self,
        height: Height,
    ) -> impl Future<Output = C::ClientState> + '_;

    /// Generates the latest consensus state for the counterparty chain.
    fn generate_counterparty_consensus_state(
        &self,
        height: Height,
    ) -> impl Future<Output = C::ConsensusState> + '_;

    fn update_counterparty_client<'a>(
        &'a self,
        counterparty: &'a C,
        counterparty_client_id: String,
        update_from: Height,
        update_to: Height,
    ) -> impl Future<Output = Height> + 'a;
}

pub trait InnerClientState {
    fn height(&self) -> Option<Height>;
}

pub trait ClientState {
    fn height(&self) -> Height;
}

impl InnerClientState for ethereum::client_state::ClientState {
    fn height(&self) -> Option<Height> {
        Some(Height {
            revision_number: 0,
            revision_height: self.latest_slot,
        })
    }
}

impl InnerClientState for cometbls::client_state::ClientState {
    fn height(&self) -> Option<Height> {
        None
    }
}

impl<Data: InnerClientState> ClientState for wasm::client_state::ClientState<Data> {
    fn height(&self) -> Height {
        self.data.height().unwrap_or(self.latest_height)
    }
}

impl<T> ClientState for Any<T>
where
    T: ClientState,
{
    fn height(&self) -> Height {
        self.0.height()
    }
}

pub struct StateProof<Data> {
    pub state: Data,
    pub proof: Vec<u8>,
    pub proof_height: Height,
}
