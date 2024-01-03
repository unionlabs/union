use std::marker::PhantomData;

use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use serde::{Deserialize, Serialize};
use unionlabs::{
    self,
    proof::{
        AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath,
        CommitmentPath, ConnectionPath, IbcPath,
    },
    traits::{ClientStateOf, ConsensusStateOf, HeaderOf, HeightOf},
};

use crate::{any_enum, fetch::FetchPacketAcknowledgement, ChainExt};

any_enum! {
    /// Data that will likely be used in a [`RelayerMsg::Aggregate`].
    #[any = AnyData]
    pub enum Data<Hc: ChainExt, Tr: ChainExt> {
        SelfClientState(SelfClientState<Hc, Tr>),
        SelfConsensusState(SelfConsensusState<Hc, Tr>),

        PacketAcknowledgement(PacketAcknowledgement<Hc, Tr>),

        ClientStateProof(IbcProof<Hc, Tr, ClientStatePath<Hc::ClientId>>),
        ClientConsensusStateProof(IbcProof<Hc, Tr, ClientConsensusStatePath<Hc::ClientId, Tr::Height>>),
        ConnectionProof(IbcProof<Hc, Tr, ConnectionPath>),
        ChannelEndProof(IbcProof<Hc, Tr, ChannelEndPath>),
        CommitmentProof(IbcProof<Hc, Tr, CommitmentPath>),
        AcknowledgementProof(IbcProof<Hc, Tr, AcknowledgementPath>),

        ClientState(IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>),
        ClientConsensusState(IbcState<Hc, Tr, ClientConsensusStatePath<Hc::ClientId, Tr::Height>>),
        Connection(IbcState<Hc, Tr, ConnectionPath>),
        ChannelEnd(IbcState<Hc, Tr, ChannelEndPath>),
        Commitment(IbcState<Hc, Tr, CommitmentPath>),
        Acknowledgement(IbcState<Hc, Tr, AcknowledgementPath>),

        #[serde(untagged)]
        LightClientSpecific(LightClientSpecificData<Hc, Tr>),
    }
}

impl<Hc: ChainExt, Tr: ChainExt> std::fmt::Display for Data<Hc, Tr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::SelfClientState(_) => write!(f, "SelfClientState"),
            Data::SelfConsensusState(_) => write!(f, "SelfConsensusState"),
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

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct SelfClientState<Hc: ChainExt, Tr: ChainExt> {
    pub self_client_state: ClientStateOf<Hc>,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> (Hc, Tr)>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct SelfConsensusState<Hc: ChainExt, Tr: ChainExt> {
    pub self_consensus_state: ConsensusStateOf<Hc>,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> (Hc, Tr)>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct Header<Hc: ChainExt, Tr: ChainExt> {
    pub header: HeaderOf<Hc>,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> (Hc, Tr)>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct IbcState<Hc: ChainExt, Tr: ChainExt, P: IbcPath<Hc, Tr>> {
    pub path: P,
    pub height: HeightOf<Hc>,
    pub state: P::Output,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct IbcProof<Hc: ChainExt, Tr: ChainExt, P: IbcPath<Hc, Tr>> {
    pub path: P,
    pub height: HeightOf<Hc>,
    pub proof: Hc::StateProof,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct PacketAcknowledgement<Hc: ChainExt, Tr: ChainExt> {
    pub fetched_by: FetchPacketAcknowledgement<Hc, Tr>,
    pub ack: Vec<u8>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct LightClientSpecificData<Hc: ChainExt, Tr: ChainExt>(pub Hc::Data<Tr>);
