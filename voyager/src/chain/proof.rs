use std::fmt::{Debug, Display};

use futures::Future;
use serde::Serialize;
use unionlabs::ibc::core::{
    channel::channel::Channel, client::height::Height, connection::connection_end::ConnectionEnd,
};

use crate::chain::{ClientStateOf, ConsensusStateOf, LightClient};

pub trait IbcStateRead<L: LightClient, P: IbcPath>
where
    StateProof<P::Output<L>>: Debug + Serialize,
{
    fn state_proof(
        light_client: &L,
        path: P,
        at: Height,
    ) -> impl Future<Output = StateProof<P::Output<L>>> + '_;
}

/// `IbcPath` represents the path to a light client's ibc storage. The values stored at each path
/// are strongly typed, i.e. `connections/{connection_id}` always stores a [`ConnectionEnd`].
pub trait IbcPath: Display + Clone + Sized {
    type Output<L: LightClient>: Debug + Serialize;
}

type ClientId = String;
type ChannelId = String;
type ConnectionId = String;
type PortId = String;

#[derive(Debug, Serialize)]
pub struct StateProof<Data> {
    pub state: Data,
    #[serde(with = "::serde_utils::hex_string")]
    pub proof: Vec<u8>,
    pub proof_height: Height,
}

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
            #[derive(Debug, Clone, PartialEq, clap::Args)]
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

        impl<T, L: LightClient> IbcStateReadPaths<L> for T
            where
                T: $(IbcStateRead<L, $Struct>+)+
        {}
    }
);

ibc_paths! {
    #[display("clients/{client_id}/clientState")]
    #[output(ClientStateOf<L::CounterpartyChain>)]
    pub struct ClientStatePath {
        pub client_id: ClientId,
    }

    #[display("clients/{client_id}/consensusStates/{height}")]
    #[output(ConsensusStateOf<L::CounterpartyChain>)]
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
