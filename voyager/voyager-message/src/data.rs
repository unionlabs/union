use std::marker::PhantomData;

use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use serde::{Deserialize, Serialize};
use unionlabs::{
    self,
    proof::{
        AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath,
        CommitmentPath, ConnectionPath, IbcPath,
    },
    traits::{ClientIdOf, ClientStateOf, ConsensusStateOf, HeaderOf, HeightOf},
};

use crate::{
    any_enum, fetch::FetchPacketAcknowledgement, identified, AnyLightClientIdentified, ChainExt,
};

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

macro_rules! data_msg {
    ($($Ty:ident,)+) => {
        $(
            impl<Hc: ChainExt, Tr: ChainExt> From<crate::Identified<Hc, Tr, $Ty<Hc, Tr>>> for crate::AnyLightClientIdentified<crate::data::AnyData>
            where
                $Ty<Hc, Tr>: Into<Data<Hc, Tr>>,
                crate::AnyLightClientIdentified<crate::data::AnyData>: From<identified!(Data<Hc, Tr>)>,
            {
                fn from(crate::Identified { chain_id, t, __marker: _ }: identified!($Ty<Hc, Tr>)) -> Self {
                    Self::from(crate::Identified::new(
                        chain_id,
                        Data::from(t),
                    ))
                }
            }

            impl<Hc: ChainExt, Tr: ChainExt> TryFrom<AnyLightClientIdentified<AnyData>>
                for crate::Identified<Hc, Tr, $Ty<Hc, Tr>>
            where
                identified!(Data<Hc, Tr>): TryFrom<
                        crate::AnyLightClientIdentified<AnyData>,
                        Error = crate::AnyLightClientIdentified<AnyData>,
                    > + Into<crate::AnyLightClientIdentified<AnyData>>,
            {
                type Error = AnyLightClientIdentified<AnyData>;

                fn try_from(value: crate::AnyLightClientIdentified<AnyData>) -> Result<Self, Self::Error> {
                    let crate::Identified { chain_id, t, __marker: _ } =
                        <crate::Identified<Hc, Tr, Data<Hc, Tr>>>::try_from(value)?;

                    Ok(crate::Identified::new(
                        chain_id.clone(),
                        <$Ty<Hc, Tr>>::try_from(t).map_err(|x: Data<Hc, Tr>| {
                            Into::<AnyLightClientIdentified<_>>::into(crate::Identified::new(chain_id, x))
                        })?,
                    ))
                }
            }
        )+
    };
}

data_msg! {
    SelfClientState,
    SelfConsensusState,

    PacketAcknowledgement,

    ClientState,
    ClientConsensusState,
    Connection,
    ChannelEnd,
    Commitment,
    Acknowledgement,

    ClientStateProof,
    ClientConsensusStateProof,
    ConnectionProof,
    ChannelEndProof,
    CommitmentProof,
    AcknowledgementProof,

    LightClientSpecificData,
}

// these are just bc im too lazy to fix the above macro
type ClientStateProof<Hc, Tr> = IbcProof<Hc, Tr, ClientStatePath<ClientIdOf<Hc>>>;
type ClientConsensusStateProof<Hc, Tr> =
    IbcProof<Hc, Tr, ClientConsensusStatePath<ClientIdOf<Hc>, HeightOf<Tr>>>;
type ConnectionProof<Hc, Tr> = IbcProof<Hc, Tr, ConnectionPath>;
type ChannelEndProof<Hc, Tr> = IbcProof<Hc, Tr, ChannelEndPath>;
type CommitmentProof<Hc, Tr> = IbcProof<Hc, Tr, CommitmentPath>;
type AcknowledgementProof<Hc, Tr> = IbcProof<Hc, Tr, AcknowledgementPath>;

type ClientState<Hc, Tr> = IbcState<Hc, Tr, ClientStatePath<ClientIdOf<Hc>>>;
type ClientConsensusState<Hc, Tr> =
    IbcState<Hc, Tr, ClientConsensusStatePath<ClientIdOf<Hc>, HeightOf<Tr>>>;
type Connection<Hc, Tr> = IbcState<Hc, Tr, ConnectionPath>;
type ChannelEnd<Hc, Tr> = IbcState<Hc, Tr, ChannelEndPath>;
type Commitment<Hc, Tr> = IbcState<Hc, Tr, CommitmentPath>;
type Acknowledgement<Hc, Tr> = IbcState<Hc, Tr, AcknowledgementPath>;
