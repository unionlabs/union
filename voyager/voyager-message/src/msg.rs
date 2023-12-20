use std::{fmt::Display, marker::PhantomData};

use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use serde::{Deserialize, Serialize};
use unionlabs::{
    ibc::core::{
        channel::{
            msg_acknowledgement::MsgAcknowledgement, msg_channel_open_ack::MsgChannelOpenAck,
            msg_channel_open_confirm::MsgChannelOpenConfirm,
            msg_channel_open_init::MsgChannelOpenInit, msg_channel_open_try::MsgChannelOpenTry,
            msg_recv_packet::MsgRecvPacket,
        },
        client::{msg_create_client::MsgCreateClient, msg_update_client::MsgUpdateClient},
        connection::{
            msg_connection_open_ack::MsgConnectionOpenAck,
            msg_connection_open_confirm::MsgConnectionOpenConfirm,
            msg_connection_open_init::MsgConnectionOpenInit,
            msg_connection_open_try::MsgConnectionOpenTry,
        },
    },
    traits::{ClientIdOf, ClientStateOf, ConsensusStateOf, HeaderOf, HeightOf},
};

use crate::{any_enum, ChainExt};

any_enum! {
    /// Defines messages that are sent *to* the lightclient `L`.
    #[any = AnyMsg]
    pub enum Msg<Hc: ChainExt, Tr: ChainExt> {
        ConnectionOpenInit(MsgConnectionOpenInitData<Hc, Tr>),
        ConnectionOpenTry(MsgConnectionOpenTryData<Hc, Tr>),
        ConnectionOpenAck(MsgConnectionOpenAckData<Hc, Tr>),
        ConnectionOpenConfirm(MsgConnectionOpenConfirmData<Hc, Tr>),

        ChannelOpenInit(MsgChannelOpenInitData<Hc, Tr>),
        ChannelOpenTry(MsgChannelOpenTryData<Hc, Tr>),
        ChannelOpenAck(MsgChannelOpenAckData<Hc, Tr>),
        ChannelOpenConfirm(MsgChannelOpenConfirmData<Hc, Tr>),

        RecvPacket(MsgRecvPacketData<Hc, Tr>),
        AckPacket(MsgAckPacketData<Hc, Tr>),

        CreateClient(MsgCreateClientData<Hc, Tr>),
        UpdateClient(MsgUpdateClientData<Hc, Tr>),
    }
}

impl<Hc: ChainExt, Tr: ChainExt> Display for Msg<Hc, Tr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Msg::ConnectionOpenInit(_) => write!(f, "ConnectionOpenInit"),
            Msg::ConnectionOpenTry(_) => write!(f, "ConnectionOpenTry"),
            Msg::ConnectionOpenAck(_) => write!(f, "ConnectionOpenAck"),
            Msg::ConnectionOpenConfirm(_) => write!(f, "ConnectionOpenConfirm"),
            Msg::ChannelOpenInit(_) => write!(f, "ChannelOpenInit"),
            Msg::ChannelOpenTry(_) => write!(f, "ChannelOpenTry"),
            Msg::ChannelOpenAck(_) => write!(f, "ChannelOpenAck"),
            Msg::ChannelOpenConfirm(_) => write!(f, "ChannelOpenConfirm"),
            Msg::RecvPacket(_) => write!(f, "RecvPacket"),
            Msg::AckPacket(_) => write!(f, "AckPacket"),
            Msg::CreateClient(_) => write!(f, "CreateClient"),
            Msg::UpdateClient(_) => write!(f, "UpdateClient"),
        }
    }
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MsgConnectionOpenInitData<Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgConnectionOpenInit<ClientIdOf<Hc>, ClientIdOf<Tr>>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MsgConnectionOpenTryData<Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgConnectionOpenTry<
        Tr::StoredClientState<Hc>,
        ClientIdOf<Hc>,
        ClientIdOf<Tr>,
        HeightOf<Tr>,
        HeightOf<Hc>,
        Tr::StateProof,
        Tr::StateProof,
        Tr::StateProof,
    >,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MsgConnectionOpenAckData<Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgConnectionOpenAck<
        Tr::StoredClientState<Hc>,
        Tr::StateProof,
        Tr::StateProof,
        Tr::StateProof,
    >,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MsgConnectionOpenConfirmData<Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgConnectionOpenConfirm<HeightOf<Tr>, Tr::StateProof>,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> Hc>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MsgChannelOpenInitData<Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgChannelOpenInit,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> (Hc, Tr)>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MsgChannelOpenTryData<Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgChannelOpenTry<Tr::StateProof>,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> (Hc, Tr)>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MsgChannelOpenAckData<Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgChannelOpenAck<Tr::StateProof>,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> (Hc, Tr)>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MsgChannelOpenConfirmData<Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgChannelOpenConfirm<Tr::StateProof>,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> (Hc, Tr)>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MsgRecvPacketData<Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgRecvPacket<Tr::StateProof>,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> (Hc, Tr)>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MsgAckPacketData<Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgAcknowledgement<Tr::StateProof>,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> (Hc, Tr)>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MsgCreateClientData<Hc: ChainExt, Tr: ChainExt> {
    pub config: <Hc as ChainExt>::Config,
    pub msg: MsgCreateClient<ClientStateOf<Tr>, ConsensusStateOf<Tr>>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MsgUpdateClientData<Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgUpdateClient<ClientIdOf<Hc>, HeaderOf<Tr>>,
    // REVIEW: Remove this field? It's currently unused
    pub update_from: HeightOf<Tr>,
}
