use futures::{Future, Stream};
use ibc_types::ibc::{
    core::{
        channel::{
            msg_channel_open_ack::MsgChannelOpenAck,
            msg_channel_open_confirm::MsgChannelOpenConfirm,
            msg_channel_open_init::MsgChannelOpenInit, msg_channel_open_try::MsgChannelOpenTry,
            msg_recv_packet::MsgRecvPacket, packet::Packet,
        },
        client::height::Height,
        connection::{
            msg_channel_open_ack::MsgConnectionOpenAck,
            msg_channel_open_confirm::MsgConnectionOpenConfirm,
            msg_channel_open_init::MsgConnectionOpenInit,
            msg_channel_open_try::MsgConnectionOpenTry,
        },
    },
    google::protobuf::any::Any,
    lightclients::{cometbls, ethereum, wasm},
};

use crate::chain::proof::{IbcStateRead, IbcStateReadPaths};

pub mod cosmos;
pub mod evm;

pub mod proof {
    use std::fmt::Display;

    use futures::Future;
    use ibc_types::ibc::core::{
        channel::channel::Channel, client::height::Height,
        connection::connection_end::ConnectionEnd,
    };

    use crate::chain::{LightClient, StateProof};

    pub trait IbcStateRead<L: LightClient, P: IbcPath> {
        fn state_proof(
            light_client: &L,
            path: P,
            at: Height,
        ) -> impl Future<Output = StateProof<P::Output<L>>> + '_;
    }

    /// `IbcPath` represents the path to a light client's ibc storage. The values stored at each path
    /// are strongly typed, i.e. `connections/{connection_id}` always stores a [`ConnectionEnd`].
    pub trait IbcPath: Display + Clone + Sized {
        type Output<L: LightClient>;
    }

    type ClientId = String;
    type ChannelId = String;
    type ConnectionId = String;
    type PortId = String;

    macro_rules! ibc_paths (
        (
            $(
                #[display($fmt:literal)]
                #[output($Output:ty)]
                pub struct $Struct:ident {
                    $(pub $field:ident: $field_ty:ty,)+
                }
            )+
        ) => {
            $(
                #[derive(Debug, Clone, PartialEq)]
                pub struct $Struct {
                    $(pub $field: $field_ty,)+
                }

                impl Display for $Struct {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        let Self { $($field,)+ } = self;
                        write!(f, $fmt)
                    }
                }

                impl IbcPath for $Struct {
                    type Output<L: LightClient> = $Output;
                }
            )+

            pub trait IbcStateReadPaths<L: LightClient>: $(IbcStateRead<L, $Struct>+)+ {}
            impl<T, L: LightClient> IbcStateReadPaths<L> for T where T: $(IbcStateRead<L, $Struct>+)+ {}
        }
    );

    ibc_paths! {
        #[display("clients/{client_id}/clientState")]
        #[output(L::ClientState)]
        pub struct ClientStatePath {
            pub client_id: ClientId,
        }

        #[display("clients/{client_id}/consensusStates/{height}")]
        #[output(L::ConsensusState)]
        pub struct ClientConsensusStatePath {
            pub client_id: ClientId,
            pub height: Height,
        }

        // #[display("clients/{client_id}/connections")]
        // pub struct ClientConnectionPath {
        //     pub client_id: ClientId,
        // }

        #[display("connections/{connection_id}")]
        #[output(ConnectionEnd)]
        pub struct ConnectionPath {
            pub connection_id: ConnectionId,
        }

        // #[display("ports/{port_id}")]
        // pub struct PortPath {
        //     pub port_id: PortId,
        // }

        #[display("channelEnds/ports/{port_id}/channels/{channel_id}")]
        #[output(Channel)]
        pub struct ChannelEndPath {
            pub port_id: PortId,
            pub channel_id: ChannelId,
        }

        // #[display("nextSequenceSend/ports/{_0}/channels/{_1}")]
        // pub struct SeqSendPath {
        //     pub port_id: PortId,
        //     pub channel_id: ChannelId,
        // }

        // #[display("nextSequenceRecv/ports/{_0}/channels/{_1}")]
        // pub struct SeqRecvPath {
        //     pub port_id: PortId,
        //     pub channel_id: ChannelId,
        // }

        // #[display("nextSequenceAck/ports/{_0}/channels/{_1}")]
        // pub struct SeqAckPath {
        //     pub port_id: PortId,
        //     pub channel_id: ChannelId,
        // }

        #[display("commitments/ports/{port_id}/channels/{channel_id}/sequences/{sequence}")]
        #[output([u8; 32])]
        pub struct CommitmentPath {
            pub port_id: PortId,
            pub channel_id: ChannelId,
            pub sequence: u64,
        }

        // #[display("acks/ports/{port_id}/channels/{channel_id}/sequences/{sequence}")]
        // pub struct AckPath {
        //     pub port_id: PortId,
        //     pub channel_id: ChannelId,
        //     pub sequence: Sequence,
        // }

        // #[display("receipts/ports/{port_id}/channels/{channel_id}/sequences/{sequence}")]
        // pub struct ReceiptPath {
        //     pub port_id: PortId,
        //     pub channel_id: ChannelId,
        //     pub sequence: Sequence,
        // }
    }
}

pub trait LightClient: Send + Sync + Sized {
    /// The client state type that this light client stores about the counterparty.
    type ClientState: ClientState;

    /// The consensus state type that this light client stores about the counterparty.
    type ConsensusState;

    type UpdateClientMessage;

    type IbcStateRead: IbcStateReadPaths<Self>;

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

    fn state_proof<P: proof::IbcPath + 'static>(
        &self,
        path: P,
        self_height: Height,
    ) -> impl Future<Output = StateProof<P::Output<Self>>> + '_
    where
        Self::IbcStateRead: IbcStateRead<Self, P>,
    {
        async move {
            <Self::IbcStateRead as IbcStateRead<Self, P>>::state_proof(self, path, self_height)
                .await
        }
    }

    fn query_latest_height(&self) -> impl Future<Output = Height> + '_;

    fn query_client_state(&self, client_id: String)
        -> impl Future<Output = Self::ClientState> + '_;

    fn process_height_for_counterparty(&self, height: Height) -> impl Future<Output = Height> + '_;

    fn packet_stream(&self)
        -> impl Future<Output = impl Stream<Item = (Height, Packet)> + '_> + '_;
}

#[derive(Debug)]
pub enum QueryHeight {
    Latest,
    Specific(Height),
}

pub trait Connect<L>: LightClient
where
    L: LightClient,
{
    // fn generate_counterparty_handshake_client_state(
    //     &self,
    //     self_state: C::ClientState,
    // ) -> impl Future<Output = C::ClientState> + '_;

    // CONNECTION HANDSHAKE

    fn connection_open_init(
        &self,
        _: MsgConnectionOpenInit,
    ) -> impl Future<Output = (String, Height)> + '_;

    fn connection_open_try(
        &self,
        _: MsgConnectionOpenTry<L::ClientState>,
    ) -> impl Future<Output = (String, Height)> + '_;

    fn connection_open_ack(
        &self,
        _: MsgConnectionOpenAck<L::ClientState>,
    ) -> impl Future<Output = Height> + '_;

    fn connection_open_confirm(
        &self,
        _: MsgConnectionOpenConfirm,
    ) -> impl Future<Output = Height> + '_;

    // CHANNEL HANDSHAKE

    fn channel_open_init(
        &self,
        _: MsgChannelOpenInit,
    ) -> impl Future<Output = (String, Height)> + '_;

    fn channel_open_try(&self, _: MsgChannelOpenTry)
        -> impl Future<Output = (String, Height)> + '_;

    fn channel_open_ack(&self, _: MsgChannelOpenAck) -> impl Future<Output = Height> + '_;

    fn channel_open_confirm(&self, _: MsgChannelOpenConfirm) -> impl Future<Output = Height> + '_;

    // PACKETS

    fn recv_packet(&self, _: MsgRecvPacket) -> impl Future<Output = ()> + Send + '_;

    // OTHER STUFF

    /// Generates the latest client state for the counterparty chain.
    fn generate_counterparty_client_state(
        &self,
        height: Height,
    ) -> impl Future<Output = L::ClientState> + '_;

    /// Generates the latest consensus state for the counterparty chain.
    fn generate_counterparty_consensus_state(
        &self,
        height: Height,
    ) -> impl Future<Output = L::ConsensusState> + '_;

    fn update_counterparty_client<'a>(
        &'a self,
        counterparty: &'a L,
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
