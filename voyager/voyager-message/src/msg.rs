use std::{fmt::Display, marker::PhantomData};

use chain_utils::{cosmos::Cosmos, evm::Evm, union::Union};
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use queue_msg::{HandleMsg, QueueMsgTypes};
use serde::{Deserialize, Serialize};
use unionlabs::{
    ethereum::config::{Mainnet, Minimal},
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

use crate::{any_enum, AnyLightClientIdentified, ChainExt, DoMsg, GetChain, RelayerMsgTypes, Wasm};

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

impl HandleMsg<RelayerMsgTypes> for AnyLightClientIdentified<AnyMsg> {
    async fn handle(
        self,
        store: &<RelayerMsgTypes as QueueMsgTypes>::Store,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            AnyLightClientIdentified::EvmMainnetOnUnion(msg) => {
                GetChain::<Wasm<Union>>::get_chain(store, &msg.chain_id)
                    .msg(msg.t)
                    .await?;
            }
            AnyLightClientIdentified::EvmMinimalOnUnion(msg) => {
                GetChain::<Wasm<Union>>::get_chain(store, &msg.chain_id)
                    .msg(msg.t)
                    .await?;
            }
            AnyLightClientIdentified::UnionOnEvmMainnet(msg) => {
                GetChain::<Evm<Mainnet>>::get_chain(store, &msg.chain_id)
                    .msg(msg.t)
                    .await?;
            }
            AnyLightClientIdentified::UnionOnEvmMinimal(msg) => {
                GetChain::<Evm<Minimal>>::get_chain(store, &msg.chain_id)
                    .msg(msg.t)
                    .await?;
            }
            AnyLightClientIdentified::CosmosOnUnion(msg) => {
                GetChain::<Union>::get_chain(store, &msg.chain_id)
                    .msg(msg.t)
                    .await?;
            }
            AnyLightClientIdentified::UnionOnCosmos(msg) => {
                GetChain::<Wasm<Cosmos>>::get_chain(store, &msg.chain_id)
                    .msg(msg.t)
                    .await?;
            }
        }

        Ok(())
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
#[serde(bound(serialize = "", deserialize = ""), transparent)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "Hc: ChainExt, Tr: ChainExt")
)]
pub struct MsgConnectionOpenInitData<Hc: ChainExt, Tr: ChainExt>(
    pub MsgConnectionOpenInit<ClientIdOf<Hc>, ClientIdOf<Tr>>,
);

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), transparent)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "Hc: ChainExt, Tr: ChainExt")
)]
pub struct MsgConnectionOpenTryData<Hc: ChainExt, Tr: ChainExt>(
    pub  MsgConnectionOpenTry<
        Tr::StoredClientState<Hc>,
        ClientIdOf<Hc>,
        ClientIdOf<Tr>,
        Tr::Height,
        Hc::Height,
        Tr::StateProof,
        Tr::StateProof,
        Tr::StateProof,
    >,
);

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), transparent)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "Hc: ChainExt, Tr: ChainExt")
)]
pub struct MsgConnectionOpenAckData<Hc: ChainExt, Tr: ChainExt>(
    pub  MsgConnectionOpenAck<
        Tr::StoredClientState<Hc>,
        Tr::StateProof,
        Tr::StateProof,
        Tr::StateProof,
    >,
);

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), transparent)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "Hc: ChainExt, Tr: ChainExt")
)]
pub struct MsgConnectionOpenConfirmData<Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgConnectionOpenConfirm<HeightOf<Tr>, Tr::StateProof>,
    #[serde(skip)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub __marker: PhantomData<fn() -> Hc>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), transparent)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "Hc: ChainExt, Tr: ChainExt")
)]
pub struct MsgChannelOpenInitData<Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgChannelOpenInit,
    #[serde(skip)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub __marker: PhantomData<fn() -> (Hc, Tr)>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), transparent)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "Hc: ChainExt, Tr: ChainExt")
)]
pub struct MsgChannelOpenTryData<Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgChannelOpenTry<Tr::StateProof>,
    #[serde(skip)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub __marker: PhantomData<fn() -> (Hc, Tr)>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), transparent)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "Hc: ChainExt, Tr: ChainExt")
)]
pub struct MsgChannelOpenAckData<Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgChannelOpenAck<Tr::StateProof, Tr::Height>,
    #[serde(skip)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub __marker: PhantomData<fn() -> (Hc, Tr)>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), transparent)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "Hc: ChainExt, Tr: ChainExt")
)]
pub struct MsgChannelOpenConfirmData<Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgChannelOpenConfirm<Tr::StateProof>,
    #[serde(skip)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub __marker: PhantomData<fn() -> (Hc, Tr)>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), transparent)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "Hc: ChainExt, Tr: ChainExt")
)]
pub struct MsgRecvPacketData<Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgRecvPacket<Tr::StateProof, Tr::Height>,
    #[serde(skip)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub __marker: PhantomData<fn() -> (Hc, Tr)>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), transparent)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "Hc: ChainExt, Tr: ChainExt")
)]
pub struct MsgAckPacketData<Hc: ChainExt, Tr: ChainExt> {
    pub msg: MsgAcknowledgement<Tr::StateProof, Tr::Height>,
    #[serde(skip)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub __marker: PhantomData<fn() -> (Hc, Tr)>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "Hc: ChainExt, Tr: ChainExt")
)]
pub struct MsgCreateClientData<Hc: ChainExt, Tr: ChainExt> {
    pub config: Hc::Config,
    pub msg: MsgCreateClient<ClientStateOf<Tr>, ConsensusStateOf<Tr>>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), transparent)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "Hc: ChainExt, Tr: ChainExt")
)]
pub struct MsgUpdateClientData<Hc: ChainExt, Tr: ChainExt>(
    pub MsgUpdateClient<ClientIdOf<Hc>, HeaderOf<Tr>>,
);
