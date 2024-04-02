use macros::apply;
use queue_msg::{data, msg_struct, HandleData, QueueError, QueueMsg, QueueMsgTypes};
use unionlabs::{
    proof::{
        AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath,
        CommitmentPath, ConnectionPath, IbcPath,
    },
    traits::{ClientStateOf, ConsensusStateOf, HeaderOf, HeightOf},
};

use crate::{
    any_enum, fetch::FetchPacketAcknowledgement, AnyLightClientIdentified, ChainExt,
    RelayMessageTypes,
};

#[apply(any_enum)]
/// Data that will likely be used in a [`QueueMsg::Aggregate`].
#[any = AnyData]
#[specific = LightClientSpecificData]
pub enum Data<Hc: ChainExt, Tr: ChainExt> {
    SelfClientState(SelfClientState<Hc, Tr>),
    SelfConsensusState(SelfConsensusState<Hc, Tr>),

    LatestHeight(LatestHeight<Hc, Tr>),

    PacketAcknowledgement(PacketAcknowledgement<Hc, Tr>),

    ClientStateProof(IbcProof<ClientStatePath<Hc::ClientId>, Hc, Tr>),
    ClientConsensusStateProof(IbcProof<ClientConsensusStatePath<Hc::ClientId, Tr::Height>, Hc, Tr>),
    ConnectionProof(IbcProof<ConnectionPath, Hc, Tr>),
    ChannelEndProof(IbcProof<ChannelEndPath, Hc, Tr>),
    CommitmentProof(IbcProof<CommitmentPath, Hc, Tr>),
    AcknowledgementProof(IbcProof<AcknowledgementPath, Hc, Tr>),

    ClientState(IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>),
    ClientConsensusState(IbcState<ClientConsensusStatePath<Hc::ClientId, Tr::Height>, Hc, Tr>),
    Connection(IbcState<ConnectionPath, Hc, Tr>),
    ChannelEnd(IbcState<ChannelEndPath, Hc, Tr>),
    Commitment(IbcState<CommitmentPath, Hc, Tr>),
    Acknowledgement(IbcState<AcknowledgementPath, Hc, Tr>),

    #[serde(untagged)]
    LightClientSpecific(LightClientSpecificData<Hc, Tr>),
}

// Passthrough since we don't want to handle any top-level data, just bubble it up to the top level.
impl HandleData<RelayMessageTypes> for AnyLightClientIdentified<AnyData> {
    fn handle(
        self,
        _: &<RelayMessageTypes as QueueMsgTypes>::Store,
    ) -> Result<QueueMsg<RelayMessageTypes>, QueueError> {
        Ok(data(self))
    }
}

impl<Hc: ChainExt, Tr: ChainExt> std::fmt::Display for Data<Hc, Tr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::SelfClientState(_) => write!(f, "SelfClientState"),
            Data::SelfConsensusState(_) => write!(f, "SelfConsensusState"),
            Data::LatestHeight(LatestHeight { height, .. }) => write!(f, "LatestHeight({height})"),
            Data::PacketAcknowledgement(_) => write!(f, "PacketAcknowledgement"),
            Data::ClientStateProof(_) => write!(f, "ClientStateProof"),
            Data::ClientConsensusStateProof(_) => write!(f, "ClientConsensusStateProof"),
            Data::ConnectionProof(_) => write!(f, "ConnectionProof"),
            Data::ChannelEndProof(_) => write!(f, "ChannelEndProof"),
            Data::CommitmentProof(_) => write!(f, "CommitmentProof"),
            Data::AcknowledgementProof(_) => write!(f, "AcknowledgementProof"),
            Data::ClientState(_) => write!(f, "ClientState"),
            Data::ClientConsensusState(_) => write!(f, "ClientConsensusState"),
            Data::Connection(_) => write!(f, "Connection"),
            Data::ChannelEnd(_) => write!(f, "ChannelEnd"),
            Data::Commitment(_) => write!(f, "Commitment"),
            Data::Acknowledgement(_) => write!(f, "Acknowledgement"),
            Data::LightClientSpecific(data) => write!(f, "LightClientSpecific({})", data.0),
        }
    }
}

#[msg_struct]
pub struct SelfClientState<Hc: ChainExt, #[cover] Tr: ChainExt> {
    pub self_client_state: ClientStateOf<Hc>,
}

#[msg_struct]
pub struct SelfConsensusState<Hc: ChainExt, #[cover] Tr: ChainExt> {
    pub self_consensus_state: ConsensusStateOf<Hc>,
}

#[msg_struct]
pub struct LatestHeight<Hc: ChainExt, #[cover] Tr: ChainExt> {
    pub height: HeightOf<Hc>,
}

#[msg_struct]
pub struct Header<Hc: ChainExt, #[cover] Tr: ChainExt> {
    pub header: HeaderOf<Hc>,
}

#[msg_struct]
pub struct IbcState<P: IbcPath<Hc, Tr>, Hc: ChainExt, Tr: ChainExt> {
    pub path: P,
    pub height: HeightOf<Hc>,
    pub state: P::Output,
}

#[msg_struct]
pub struct IbcProof<P: IbcPath<Hc, Tr>, Hc: ChainExt, #[cover] Tr: ChainExt> {
    pub path: P,
    pub height: HeightOf<Hc>,
    pub proof: Hc::StateProof,
}

#[msg_struct]
pub struct PacketAcknowledgement<Hc: ChainExt, Tr: ChainExt> {
    pub fetched_by: FetchPacketAcknowledgement<Hc, Tr>,
    pub ack: Vec<u8>,
}

#[msg_struct]
pub struct LightClientSpecificData<Hc: ChainExt, Tr: ChainExt>(pub Hc::Data<Tr>);
