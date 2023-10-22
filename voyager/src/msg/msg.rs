use std::{fmt::Display, marker::PhantomData};

use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use serde::{Deserialize, Serialize};
use unionlabs::ibc::core::{
    channel::{
        msg_acknowledgement::MsgAcknowledgement, msg_channel_open_ack::MsgChannelOpenAck,
        msg_channel_open_confirm::MsgChannelOpenConfirm, msg_channel_open_init::MsgChannelOpenInit,
        msg_channel_open_try::MsgChannelOpenTry, msg_recv_packet::MsgRecvPacket,
    },
    client::{msg_create_client::MsgCreateClient, msg_update_client::MsgUpdateClient},
    connection::{
        msg_connection_open_ack::MsgConnectionOpenAck,
        msg_connection_open_confirm::MsgConnectionOpenConfirm,
        msg_connection_open_init::MsgConnectionOpenInit,
        msg_connection_open_try::MsgConnectionOpenTry,
    },
};

use crate::{
    chain::{
        ChainOf, ClientStateOf, ConsensusStateOf, HeaderOf, HeightOf, LightClient, LightClientBase,
    },
    msg::any_enum,
};

any_enum! {
    /// Defines messages that are sent *to* the lightclient `L`.
    #[any = AnyMsg]
    pub enum Msg<L: LightClient> {
        ConnectionOpenInit(MsgConnectionOpenInitData<L>),
        ConnectionOpenTry(MsgConnectionOpenTryData<L>),
        ConnectionOpenAck(MsgConnectionOpenAckData<L>),
        ConnectionOpenConfirm(MsgConnectionOpenConfirmData<L>),

        ChannelOpenInit(MsgChannelOpenInitData<L>),
        ChannelOpenTry(MsgChannelOpenTryData<L>),
        ChannelOpenAck(MsgChannelOpenAckData<L>),
        ChannelOpenConfirm(MsgChannelOpenConfirmData<L>),

        RecvPacket(MsgRecvPacketData<L>),
        AckPacket(MsgAckPacketData<L>),

        CreateClient(MsgCreateClientData<L>),
        UpdateClient(MsgUpdateClientData<L>),
    }
}

impl<L: LightClient> Display for Msg<L> {
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
pub struct MsgConnectionOpenInitData<L: LightClient> {
    pub msg: MsgConnectionOpenInit<L::ClientId, <L::Counterparty as LightClientBase>::ClientId>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MsgConnectionOpenTryData<L: LightClient> {
    pub msg: MsgConnectionOpenTry<
        ClientStateOf<L::HostChain>,
        L::ClientId,
        <L::Counterparty as LightClientBase>::ClientId,
        HeightOf<ChainOf<L::Counterparty>>,
        HeightOf<ChainOf<L>>,
    >,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MsgConnectionOpenAckData<L: LightClient> {
    pub msg: MsgConnectionOpenAck<ClientStateOf<L::HostChain>>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MsgConnectionOpenConfirmData<L: LightClient>(
    pub MsgConnectionOpenConfirm<HeightOf<ChainOf<L::Counterparty>>>,
);

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MsgChannelOpenInitData<L: LightClient> {
    pub msg: MsgChannelOpenInit,
    #[serde(skip)]
    pub __marker: PhantomData<L>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MsgChannelOpenTryData<L: LightClient> {
    pub msg: MsgChannelOpenTry,
    #[serde(skip)]
    pub __marker: PhantomData<L>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MsgChannelOpenAckData<L: LightClient> {
    pub msg: MsgChannelOpenAck,
    #[serde(skip)]
    pub __marker: PhantomData<L>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MsgChannelOpenConfirmData<L: LightClient> {
    pub msg: MsgChannelOpenConfirm,
    #[serde(skip)]
    pub __marker: PhantomData<L>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MsgRecvPacketData<L: LightClient> {
    pub msg: MsgRecvPacket,
    #[serde(skip)]
    pub __marker: PhantomData<L>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MsgAckPacketData<L: LightClient> {
    pub msg: MsgAcknowledgement,
    #[serde(skip)]
    pub __marker: PhantomData<L>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MsgCreateClientData<L: LightClient> {
    pub config: L::Config,
    pub msg: MsgCreateClient<
        ClientStateOf<ChainOf<L::Counterparty>>,
        ConsensusStateOf<ChainOf<L::Counterparty>>,
    >,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MsgUpdateClientData<L: LightClient> {
    pub msg:
        MsgUpdateClient<L::ClientId, HeaderOf<<L::Counterparty as LightClientBase>::HostChain>>,
    pub update_from: HeightOf<ChainOf<L::Counterparty>>,
}
