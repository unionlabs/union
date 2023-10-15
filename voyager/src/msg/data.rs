use std::marker::PhantomData;

use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use serde::{Deserialize, Serialize};
use unionlabs::{
    self,
    ibc::core::channel::channel::Channel,
    proof::{
        AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath,
        CommitmentPath, ConnectionPath,
    },
    traits::Chain,
};

use crate::{
    chain::{ChainOf, ClientStateOf, ConsensusStateOf, HeaderOf, HeightOf, LightClient},
    msg::{any_enum, fetch::FetchPacketAcknowledgement, identified, StateProofOf},
};

any_enum! {
    /// Data that will likely be used in a [`RelayerMsg::Aggregate`].
    #[any = AnyData(identified!(Data<L>))]
    pub enum Data<L: LightClient> {
        SelfClientState(SelfClientState<L>),
        SelfConsensusState(SelfConsensusState<L>),

        ChannelEnd(ChannelEnd<L>),
        ConnectionEnd(ConnectionEnd<L>),
        PacketAcknowledgement(PacketAcknowledgement<L>),

        TrustedClientState(TrustedClientState<L>),
        ClientStateProof(ClientStateProof<L>),
        ClientConsensusStateProof(ClientConsensusStateProof<L>),
        ConnectionProof(ConnectionProof<L>),
        ChannelEndProof(ChannelEndProof<L>),
        CommitmentProof(CommitmentProof<L>),
        AcknowledgementProof(AcknowledgementProof<L>),

        LightClientSpecific(LightClientSpecificData<L>),
    }
}

impl<L: LightClient> std::fmt::Display for Data<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::SelfClientState(_) => write!(f, "SelfClientState"),
            Data::SelfConsensusState(_) => write!(f, "SelfConsensusState"),
            Data::ChannelEnd(_) => write!(f, "ChannelEnd"),
            Data::ConnectionEnd(_) => write!(f, "ConnectionEnd"),
            Data::PacketAcknowledgement(_) => write!(f, "PacketAcknowledgement"),
            Data::TrustedClientState(_) => write!(f, "TrustedClientState"),
            Data::ClientStateProof(_) => write!(f, "ClientStateProof"),
            Data::ClientConsensusStateProof(_) => write!(f, "ClientConsensusStateProof"),
            Data::ConnectionProof(_) => write!(f, "ConnectionProof"),
            Data::ChannelEndProof(_) => write!(f, "ChannelEndProof"),
            Data::CommitmentProof(_) => write!(f, "CommitmentProof"),
            Data::AcknowledgementProof(_) => write!(f, "AcknowledgementProof"),
            Data::LightClientSpecific(data) => write!(f, "LightClientSpecific({})", data.0),
        }
    }
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct SelfClientState<L: LightClient>(pub ClientStateOf<L::HostChain>);

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct SelfConsensusState<L: LightClient>(pub ConsensusStateOf<L::HostChain>);

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct Header<L: LightClient>(pub HeaderOf<L::HostChain>);

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct ClientStateProof<L: LightClient>(
    pub StateProofOf<ClientStatePath<<ChainOf<L> as Chain>::ClientId>, L>,
);

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct ClientConsensusStateProof<L: LightClient>(
    pub  StateProofOf<
        ClientConsensusStatePath<
            <ChainOf<L> as Chain>::ClientId,
            <ChainOf<L::Counterparty> as Chain>::Height,
        >,
        L,
    >,
);

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct ConnectionProof<L: LightClient>(pub StateProofOf<ConnectionPath, L>);

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct ChannelEndProof<L: LightClient>(pub StateProofOf<ChannelEndPath, L>);

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct CommitmentProof<L: LightClient>(pub StateProofOf<CommitmentPath, L>);

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AcknowledgementProof<L: LightClient>(pub StateProofOf<AcknowledgementPath, L>);

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct ChannelEnd<L: LightClient> {
    pub channel: Channel,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> L>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct ConnectionEnd<L: LightClient>(
    pub  unionlabs::ibc::core::connection::connection_end::ConnectionEnd<
        L::ClientId,
        <L::Counterparty as LightClient>::ClientId,
        // NOTE: String used here since it may be empty; figure out a way to more strongly type this
        String,
    >,
);

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct PacketAcknowledgement<L: LightClient> {
    pub fetched_by: FetchPacketAcknowledgement<L>,
    pub ack: Vec<u8>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct TrustedClientState<L: LightClient> {
    pub fetched_at: HeightOf<ChainOf<L>>,
    pub client_id: L::ClientId,
    pub trusted_client_state: ClientStateOf<<L::Counterparty as LightClient>::HostChain>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct LightClientSpecificData<L: LightClient>(pub L::Data);

macro_rules! data_msg {
    ($($Ty:ident,)+) => {
        $(
            impl<L: LightClient> From<crate::msg::Identified<L, $Ty<L>>> for crate::msg::AggregateData
            where
                $Ty<L>: Into<Data<L>>,
                crate::msg::AggregateData: From<identified!(Data<L>)>,
            {
                fn from(crate::msg::Identified { chain_id, data }: identified!($Ty<L>)) -> Self {
                    Self::from(crate::msg::Identified {
                        chain_id,
                        data: Data::from(data),
                    })
                }
            }
        )+
    };
}

data_msg! {
    SelfClientState,
    SelfConsensusState,

    ChannelEnd,
    ConnectionEnd,
    PacketAcknowledgement,

    TrustedClientState,

    ClientStateProof,
    ClientConsensusStateProof,
    ConnectionProof,
    ChannelEndProof,
    CommitmentProof,
    AcknowledgementProof,

    LightClientSpecificData,
}
