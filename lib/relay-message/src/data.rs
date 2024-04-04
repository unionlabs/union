use macros::apply;
use queue_msg::{data, queue_msg, HandleData, QueueError, QueueMsg, QueueMsgTypes};
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

#[queue_msg]
pub struct SelfClientState<Hc: ChainExt, #[cover] Tr: ChainExt> {
    pub self_client_state: ClientStateOf<Hc>,
}

#[queue_msg]
pub struct SelfConsensusState<Hc: ChainExt, #[cover] Tr: ChainExt> {
    pub self_consensus_state: ConsensusStateOf<Hc>,
}

#[queue_msg]
pub struct LatestHeight<Hc: ChainExt, #[cover] Tr: ChainExt> {
    pub height: HeightOf<Hc>,
}

#[queue_msg]
pub struct Header<Hc: ChainExt, #[cover] Tr: ChainExt> {
    pub header: HeaderOf<Hc>,
}

#[queue_msg]
pub struct IbcState<P: IbcPath<Hc, Tr>, Hc: ChainExt, Tr: ChainExt> {
    pub path: P,
    pub height: HeightOf<Hc>,
    pub state: P::Output,
}

#[queue_msg]
pub struct IbcProof<P: IbcPath<Hc, Tr>, Hc: ChainExt, #[cover] Tr: ChainExt> {
    pub path: P,
    pub height: HeightOf<Hc>,
    pub proof: Hc::StateProof,
}

#[queue_msg]
pub struct PacketAcknowledgement<Hc: ChainExt, Tr: ChainExt> {
    pub fetched_by: FetchPacketAcknowledgement<Hc, Tr>,
    pub ack: Vec<u8>,
}

#[queue_msg]
pub struct LightClientSpecificData<Hc: ChainExt, Tr: ChainExt>(pub Hc::Data<Tr>);
