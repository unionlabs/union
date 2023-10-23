#![allow(clippy::type_complexity)]

use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
};

use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use serde::{Deserialize, Serialize};
use unionlabs::{
    proof::IbcPath,
    traits::{Chain, ClientState},
};

use crate::{
    chain::{
        evm::{CometblsMainnet, CometblsMinimal},
        proof::StateProof,
        union::{EthereumMainnet, EthereumMinimal},
        LightClient, LightClientBase,
    },
    msg::{
        aggregate::AnyAggregate,
        data::{AnyData, Data},
        event::{AnyEvent, Event},
        fetch::{AnyFetch, Fetch},
        msg::{AnyMsg, Msg},
        wait::{AnyWait, Wait},
    },
};

pub mod data;
pub mod event;
pub mod fetch;
#[allow(clippy::module_inception)] // fight me clippy
pub mod msg;
pub mod wait;

pub type ChainIdOf<L> =
    <<<L as LightClientBase>::HostChain as Chain>::SelfClientState as ClientState>::ChainId;

pub type StateProofOf<T, L> = StateProof<
    <T as IbcPath<
        <L as LightClientBase>::HostChain,
        <<L as LightClientBase>::Counterparty as LightClientBase>::HostChain,
    >>::Output,
>;

pub trait IntoRelayerMsg {
    fn into_relayer_msg(self) -> RelayerMsg;
}

pub trait TryFromRelayerMsg: Sized {
    fn try_from_relayer_msg(msg: RelayerMsg) -> Result<Self, RelayerMsg>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeferPoint {
    Absolute,
    Relative,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum RelayerMsg {
    Lc(AnyLightClientIdentified<AnyLcMsg>),
    DeferUntil {
        point: DeferPoint,
        seconds: u64,
    },
    Repeat {
        times: u64,
        msg: Box<RelayerMsg>,
    },
    Timeout {
        timeout_timestamp: u64,
        msg: Box<RelayerMsg>,
    },
    Sequence(VecDeque<RelayerMsg>),
    Retry(u8, Box<RelayerMsg>),
    Aggregate {
        /// Messages that are expected to resolve to [`Data`].
        queue: VecDeque<RelayerMsg>,
        /// The resolved data messages.
        data: VecDeque<AnyLightClientIdentified<AnyData>>,
        /// The message that will utilize the aggregated data.
        receiver: AnyLightClientIdentified<AnyAggregate>,
    },
}

impl TryFrom<AnyLightClientIdentified<AnyLcMsg>> for AnyLightClientIdentified<AnyData> {
    type Error = AnyLightClientIdentified<AnyLcMsg>;

    fn try_from(value: AnyLightClientIdentified<AnyLcMsg>) -> Result<Self, Self::Error> {
        match value {
            AnyLightClientIdentified::EthereumMainnet(i) => <Data<_>>::try_from(i.data)
                .map(|d| Identified::new(i.chain_id.clone(), d))
                .map(AnyLightClientIdentified::EthereumMainnet)
                .map_err(|l| Identified::new(i.chain_id, l))
                .map_err(AnyLightClientIdentified::EthereumMainnet),
            AnyLightClientIdentified::EthereumMinimal(i) => <Data<_>>::try_from(i.data)
                .map(|d| Identified::new(i.chain_id.clone(), d))
                .map(AnyLightClientIdentified::EthereumMinimal)
                .map_err(|l| Identified::new(i.chain_id, l))
                .map_err(AnyLightClientIdentified::EthereumMinimal),
            AnyLightClientIdentified::CometblsMainnet(i) => <Data<_>>::try_from(i.data)
                .map(|d| Identified::new(i.chain_id, d))
                .map(AnyLightClientIdentified::CometblsMainnet)
                .map_err(|l| Identified::new(i.chain_id, l))
                .map_err(AnyLightClientIdentified::CometblsMainnet),
            AnyLightClientIdentified::CometblsMinimal(i) => <Data<_>>::try_from(i.data)
                .map(|d| Identified::new(i.chain_id, d))
                .map(AnyLightClientIdentified::CometblsMinimal)
                .map_err(|l| Identified::new(i.chain_id, l))
                .map_err(AnyLightClientIdentified::CometblsMinimal),
        }
    }
}

pub type AggregateReceiver = AnyLightClientIdentified<AnyAggregate>;
pub type AggregateData = AnyLightClientIdentified<AnyData>;

impl std::fmt::Display for RelayerMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelayerMsg::Lc(lc) => write!(f, "Lc({lc})"),
            RelayerMsg::DeferUntil { point, seconds } => {
                write!(f, "DeferUntil({:?}, {seconds})", point)
            }
            RelayerMsg::Repeat { times, msg } => write!(f, "Repeat({times}, {msg})"),
            RelayerMsg::Timeout {
                timeout_timestamp,
                msg,
            } => write!(f, "Timeout({timeout_timestamp}, {msg})"),
            RelayerMsg::Sequence(seq) => {
                write!(f, "Sequence [")?;
                let len = seq.len();
                for (idx, msg) in seq.iter().enumerate() {
                    write!(f, "{msg}")?;
                    if idx != len - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
            RelayerMsg::Retry(remaining, msg) => write!(f, "Retry({remaining}, {msg})"),
            RelayerMsg::Aggregate {
                queue,
                data,
                receiver,
            } => {
                let data = data
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                let queue = queue
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                write!(f, "Aggregate([{queue}] -> [{data}] -> {receiver})")
            }
        }
    }
}

impl TryFrom<RelayerMsg> for AnyLightClientIdentified<AnyLcMsg> {
    type Error = RelayerMsg;

    fn try_from(value: RelayerMsg) -> Result<Self, Self::Error> {
        match value {
            RelayerMsg::Lc(ok) => Ok(ok),
            _ => Err(value),
        }
    }
}

impl TryFrom<RelayerMsg> for AnyLightClientIdentified<AnyData> {
    type Error = RelayerMsg;

    fn try_from(value: RelayerMsg) -> Result<Self, Self::Error> {
        match value {
            RelayerMsg::Lc(any_lc_msg) => {
                AnyLightClientIdentified::<AnyData>::try_from(any_lc_msg).map_err(RelayerMsg::Lc)
            }
            _ => Err(value),
        }
    }
}

impl From<AnyLightClientIdentified<AnyLcMsg>> for RelayerMsg {
    fn from(value: AnyLightClientIdentified<AnyLcMsg>) -> Self {
        Self::Lc(value)
    }
}

impl<L: LightClient> TryFrom<RelayerMsg> for LcMsg<L>
where
    LcMsg<L>: TryFrom<AnyLightClientIdentified<AnyLcMsg>, Error = AnyLightClientIdentified<AnyLcMsg>>
        + Into<AnyLightClientIdentified<AnyLcMsg>>,
{
    type Error = RelayerMsg;

    fn try_from(value: RelayerMsg) -> Result<Self, Self::Error> {
        LcMsg::<L>::try_from(<AnyLightClientIdentified<AnyLcMsg>>::try_from(value)?)
            .map_err(Into::into)
    }
}

impl<L: LightClient> From<LcMsg<L>> for RelayerMsg
where
    AnyLightClientIdentified<AnyLcMsg>: From<LcMsg<L>>,
{
    fn from(value: LcMsg<L>) -> Self {
        RelayerMsg::Lc(<AnyLightClientIdentified<AnyLcMsg>>::from(value))
    }
}

macro_rules! any_enum {
    (
        $(#[doc = $outer_doc:literal])*
        #[any = $Any:ident]
        pub enum $Enum:ident<L: LightClient> {
            $(
                $(#[doc = $doc:literal])*
                $Variant:ident$((
                    $(#[$variant_inner_meta:meta])*
                    $VariantInner:ty
                ))?,
            )+
        }
    ) => {
        #[derive(frame_support_procedural::DebugNoBound, frame_support_procedural::CloneNoBound, frame_support_procedural::PartialEqNoBound, serde::Serialize, serde::Deserialize)]
        #[serde(bound(serialize = "", deserialize = ""))]
        $(#[doc = $outer_doc])*
        #[allow(clippy::large_enum_variant)]
        pub enum $Enum<L: LightClient> {
            $(
                $(#[doc = $doc])*
                $Variant$((
                    $(#[$variant_inner_meta])*
                    $VariantInner
                ))?,
            )+
        }

        pub enum $Any {}
        impl crate::msg::AnyLightClient for $Any {
            type Inner<L: LightClient> = $Enum<L>;
        }

        impl<L: LightClient> TryFrom<crate::msg::LcMsg<L>> for $Enum<L> {
            type Error = crate::msg::LcMsg<L>;

            fn try_from(value: crate::msg::LcMsg<L>) -> Result<Self, Self::Error> {
                if let crate::msg::LcMsg::$Enum(t) = value {
                    Ok(t)
                } else {
                    Err(value)
                }
            }
        }

        impl<L: LightClient> From<crate::msg::Identified<L, crate::msg::InnerOf<$Any, L>>> for crate::msg::RelayerMsg
        where
            crate::msg::LcMsg<L>: From<crate::msg::InnerOf<$Any, L>>,
            crate::msg::AnyLightClientIdentified<crate::msg::AnyLcMsg>:
                From<crate::msg::Identified<L, crate::msg::InnerOf<crate::msg::AnyLcMsg, L>>>
        {
            fn from(value: crate::msg::Identified<L, crate::msg::InnerOf<$Any, L>>) -> Self {
                Self::Lc(
                    <crate::msg::AnyLightClientIdentified<crate::msg::AnyLcMsg>>::from(
                        crate::msg::Identified {
                            chain_id: value.chain_id, data: crate::msg::LcMsg::from(value.data)
                        }
                    )
                )
            }
        }

        impl<L: LightClient> TryFrom<crate::msg::RelayerMsg> for crate::msg::Identified<L, crate::msg::InnerOf<$Any, L>>
        where
            crate::msg::AnyLightClientIdentified<crate::msg::AnyLcMsg>: TryFrom<crate::msg::RelayerMsg, Error = crate::msg::RelayerMsg> + Into<crate::msg::RelayerMsg>,
            crate::msg::Identified<L, crate::msg::LcMsg<L>>: TryFrom<crate::msg::AnyLightClientIdentified<crate::msg::AnyLcMsg>, Error = crate::msg::AnyLightClientIdentified<crate::msg::AnyLcMsg>>
                + Into<crate::msg::AnyLightClientIdentified<crate::msg::AnyLcMsg>>,
            crate::msg::InnerOf<$Any, L>: TryFrom<crate::msg::LcMsg<L>, Error = crate::msg::LcMsg<L>> + Into<crate::msg::LcMsg<L>>,
        {
            type Error = crate::msg::RelayerMsg;
            fn try_from(value: crate::msg::RelayerMsg) -> Result<Self, crate::msg::RelayerMsg> {
                let any_lc_msg = <crate::msg::AnyLightClientIdentified<crate::msg::AnyLcMsg>>::try_from(value)?;
                let identified_lc_msg = <crate::msg::Identified<L, crate::msg::LcMsg<L>>>::try_from(any_lc_msg)
                    .map_err(<crate::msg::AnyLightClientIdentified<crate::msg::AnyLcMsg>>::from)?;
                let data =
                    <crate::msg::InnerOf<$Any, L>>::try_from(identified_lc_msg.data).map_err(|x: crate::msg::LcMsg<L>| {
                        Into::<crate::msg::AnyLightClientIdentified<crate::msg::AnyLcMsg>>::into(crate::msg::Identified::<L, crate::msg::LcMsg<L>>::new(
                            identified_lc_msg.chain_id.clone(),
                            x,
                        ))
                    })?;

                Ok(crate::msg::Identified::new(identified_lc_msg.chain_id, data))
            }
        }

        $(
            $(
                impl<L: LightClient> TryFrom<$Enum<L>> for $VariantInner {
                    type Error = $Enum<L>;

                    fn try_from(value: $Enum<L>) -> Result<Self, Self::Error> {
                        match value {
                            $Enum::$Variant(t) => Ok(t),
                            _ => Err(value),
                        }
                    }
                }

                impl<L: LightClient> From<$VariantInner> for $Enum<L> {
                    fn from(value: $VariantInner) -> Self {
                        Self::$Variant(value)
                    }
                }

                impl<L: LightClient> TryInto<crate::msg::Identified<L, $VariantInner>> for crate::msg::RelayerMsg
                where
                    crate::msg::AnyLightClientIdentified<crate::msg::AnyLcMsg>: TryFrom<crate::msg::RelayerMsg, Error = crate::msg::RelayerMsg> + Into<crate::msg::RelayerMsg>,
                    crate::msg::LcMsg<L>: TryFrom<crate::msg::AnyLightClientIdentified<crate::msg::AnyLcMsg>, Error = crate::msg::AnyLightClientIdentified<crate::msg::AnyLcMsg>> + Into<crate::msg::AnyLightClientIdentified<crate::msg::AnyLcMsg>>,
                    crate::msg::Identified<L, $VariantInner>: TryFrom<crate::msg::LcMsg<L>, Error = crate::msg::LcMsg<L>> + Into<crate::msg::LcMsg<L>>,
                {
                    type Error = crate::msg::RelayerMsg;

                    fn try_into(self) -> Result<crate::msg::Identified<L, $VariantInner>, crate::msg::RelayerMsg> {
                        <crate::msg::AnyLightClientIdentified<crate::msg::AnyLcMsg>>::try_from(self)
                            .and_then(|x| <crate::msg::LcMsg<L>>::try_from(x).map_err(Into::into))
                            .and_then(|x| {
                                <crate::msg::Identified<L, $VariantInner>>::try_from(x)
                                    .map_err(Into::<crate::msg::LcMsg<L>>::into)
                                    .map_err(Into::<crate::msg::AnyLightClientIdentified<crate::msg::AnyLcMsg>>::into)
                                    .map_err(Into::<crate::msg::RelayerMsg>::into)
                            })
                    }
                }

                impl<L: LightClient> TryFrom<crate::msg::LcMsg<L>> for $VariantInner {
                    type Error = crate::msg::LcMsg<L>;

                    fn try_from(value: crate::msg::LcMsg<L>) -> Result<Self, crate::msg::LcMsg<L>> {
                        match value {
                            crate::msg::LcMsg::$Enum($Enum::$Variant(data)) => Ok(data),
                            _ => Err(value),
                        }
                    }
                }

                impl<L: LightClient> TryFrom<crate::msg::AnyLightClientIdentified<crate::msg::AnyLcMsg>> for crate::msg::Identified<L, $VariantInner>
                where
                    crate::msg::LcMsg<L>: TryFrom<crate::msg::AnyLightClientIdentified<crate::msg::AnyLcMsg>, Error = crate::msg::AnyLightClientIdentified<crate::msg::AnyLcMsg>> + Into<crate::msg::AnyLightClientIdentified<crate::msg::AnyLcMsg>>,
                    Self: TryFrom<crate::msg::LcMsg<L>, Error = crate::msg::LcMsg<L>> + Into<crate::msg::LcMsg<L>>,
                {
                    type Error = crate::msg::AnyLightClientIdentified<crate::msg::AnyLcMsg>;

                    fn try_from(value: crate::msg::AnyLightClientIdentified<crate::msg::AnyLcMsg>) -> Result<Self, crate::msg::AnyLightClientIdentified<crate::msg::AnyLcMsg>> {
                        crate::msg::LcMsg::<L>::try_from(value).and_then(|x| Self::try_from(x).map_err(Into::into))
                    }
                }
            )?
        )+
    };
}

pub(crate) use any_enum;

// TODO: Move to file
pub mod aggregate {
    use std::fmt::Display;

    use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
    use serde::{Deserialize, Serialize};
    use unionlabs::{
        ethereum::H256,
        events::{
            ChannelOpenAck, ChannelOpenInit, ChannelOpenTry, ConnectionOpenAck, ConnectionOpenInit,
            ConnectionOpenTry, RecvPacket, SendPacket,
        },
    };

    use super::ChainIdOf;
    use crate::{
        chain::{ChainOf, HeightOf, LightClient, LightClientBase},
        msg::fetch::FetchStateProof,
    };

    any_enum! {
        /// Aggregate data, using data from [`AggregateData`]
        #[any = AnyAggregate]
        pub enum Aggregate<L: LightClient> {
            ConnectionOpenTry(AggregateConnectionOpenTry<L>),
            ConnectionOpenAck(AggregateConnectionOpenAck<L>),
            ConnectionOpenConfirm(AggregateConnectionOpenConfirm<L>),

            ChannelOpenTry(AggregateChannelOpenTry<L>),
            ChannelOpenAck(AggregateChannelOpenAck<L>),
            ChannelOpenConfirm(AggregateChannelOpenConfirm<L>),

            RecvPacket(AggregateRecvPacket<L>),
            AckPacket(AggregateAckPacket<L>),

            ConnectionFetchFromChannelEnd(AggregateConnectionFetchFromChannelEnd<L>),

            // Aggregate that fetches the connection info from the channel
            ChannelHandshakeUpdateClient(AggregateChannelHandshakeUpdateClient<L>),

            PacketUpdateClient(AggregatePacketUpdateClient<L>),

            WaitForTrustedHeight(AggregateWaitForTrustedHeight<L>),

            FetchCounterpartyStateproof(AggregateFetchCounterpartyStateProof<L>),

            UpdateClientFromClientId(AggregateUpdateClientFromClientId<L>),

            UpdateClient(AggregateUpdateClient<L>),
            UpdateClientWithCounterpartyChainIdData(AggregateUpdateClientWithCounterpartyChainId<L>),

            CreateClient(AggregateCreateClient<L>),

            ConsensusStateProofAtLatestHeight(ConsensusStateProofAtLatestHeight<L>),

            AggregateMsgAfterUpdate(AggregateMsgAfterUpdate<L>),

            LightClientSpecific(LightClientSpecificAggregate<L>),
        }
    }

    impl<L: LightClient> Display for Aggregate<L> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Aggregate::ConnectionOpenTry(_) => write!(f, "ConnectionOpenTry"),
                Aggregate::ConnectionOpenAck(_) => write!(f, "ConnectionOpenAck"),
                Aggregate::ConnectionOpenConfirm(_) => write!(f, "ConnectionOpenConfirm"),
                Aggregate::ChannelOpenTry(_) => write!(f, "ChannelOpenTry"),
                Aggregate::ChannelOpenAck(_) => write!(f, "ChannelOpenAck"),
                Aggregate::ChannelOpenConfirm(_) => write!(f, "ChannelOpenConfirm"),
                Aggregate::RecvPacket(_) => write!(f, "RecvPacket"),
                Aggregate::AckPacket(_) => write!(f, "AckPacket"),
                Aggregate::ConnectionFetchFromChannelEnd(_) => {
                    write!(f, "ConnectionFetchFromChannelEnd")
                }
                Aggregate::ChannelHandshakeUpdateClient(_) => {
                    write!(f, "ChannelHandshakeUpdateClient")
                }
                Aggregate::PacketUpdateClient(msg) => {
                    write!(
                        f,
                        "PacketUpdateClient::{}",
                        match msg.packet_event {
                            PacketEvent::Send(_) => "Send",
                            PacketEvent::Recv(_) => "Recv",
                        }
                    )
                }
                Aggregate::WaitForTrustedHeight(_) => write!(f, "WaitForTrustedHeight"),
                Aggregate::FetchCounterpartyStateproof(_) => {
                    write!(f, "FetchCounterpartyStateproof")
                }
                Aggregate::UpdateClientFromClientId(_) => write!(f, "UpdateClientFromClientId"),
                Aggregate::UpdateClient(_) => write!(f, "UpdateClient"),
                Aggregate::UpdateClientWithCounterpartyChainIdData(_) => {
                    write!(f, "UpdateClientWithCounterpartyChainIdData")
                }
                Aggregate::CreateClient(_) => write!(f, "CreateClient"),
                Aggregate::ConsensusStateProofAtLatestHeight(_) => {
                    write!(f, "ConsensusStateProofAtLatestHeight")
                }
                Aggregate::AggregateMsgAfterUpdate(msg) => {
                    write!(f, "AggregateMsgAfterUpdate::")?;
                    match msg {
                        AggregateMsgAfterUpdate::ConnectionOpenTry(_) => {
                            write!(f, "ConnectionOpenTry")
                        }
                        AggregateMsgAfterUpdate::ConnectionOpenAck(_) => {
                            write!(f, "ConnectionOpenAck")
                        }
                        AggregateMsgAfterUpdate::ConnectionOpenConfirm(_) => {
                            write!(f, "ConnectionOpenConfirm")
                        }
                        AggregateMsgAfterUpdate::ChannelOpenTry(_) => write!(f, "ChannelOpenTry"),
                        AggregateMsgAfterUpdate::ChannelOpenAck(_) => write!(f, "ChannelOpenAck"),
                        AggregateMsgAfterUpdate::ChannelOpenConfirm(_) => {
                            write!(f, "ChannelOpenConfirm")
                        }
                        AggregateMsgAfterUpdate::RecvPacket(_) => write!(f, "RecvPacket"),
                        AggregateMsgAfterUpdate::AckPacket(_) => write!(f, "AckPacket"),
                    }
                }
                Aggregate::LightClientSpecific(agg) => write!(f, "LightClientSpecific({})", agg.0),
            }
        }
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregateConnectionOpenTry<L: LightClient> {
        pub event_height: HeightOf<L::HostChain>,
        pub event: ConnectionOpenInit<L::ClientId, <L::Counterparty as LightClientBase>::ClientId>,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregateConnectionOpenAck<L: LightClient> {
        pub event_height: HeightOf<L::HostChain>,
        pub event: ConnectionOpenTry<L::ClientId, <L::Counterparty as LightClientBase>::ClientId>,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregateConnectionOpenConfirm<L: LightClient> {
        pub event_height: HeightOf<L::HostChain>,
        pub event: ConnectionOpenAck<L::ClientId, <L::Counterparty as LightClientBase>::ClientId>,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregateChannelOpenTry<L: LightClient> {
        pub event_height: HeightOf<L::HostChain>,
        pub event: ChannelOpenInit,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregateChannelOpenAck<L: LightClient> {
        pub event_height: HeightOf<L::HostChain>,
        pub event: ChannelOpenTry,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregateChannelOpenConfirm<L: LightClient> {
        pub event_height: HeightOf<L::HostChain>,
        pub event: ChannelOpenAck,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregateRecvPacket<L: LightClient> {
        pub event_height: HeightOf<L::HostChain>,
        pub event: SendPacket,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregateAckPacket<L: LightClient> {
        pub event_height: HeightOf<L::HostChain>,
        pub event: RecvPacket,
        // HACK: Need to pass the block hash through, figure out a better/cleaner way to do this
        pub block_hash: H256,
        pub counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregateConnectionFetchFromChannelEnd<L: LightClient> {
        pub at: HeightOf<ChainOf<L>>,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregateChannelHandshakeUpdateClient<L: LightClient> {
        // Will be threaded through to the update msg
        pub update_to: HeightOf<L::HostChain>,
        pub event_height: HeightOf<L::HostChain>,
        pub channel_handshake_event: ChannelHandshakeEvent,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub enum ChannelHandshakeEvent {
        Init(ChannelOpenInit),
        Try(ChannelOpenTry),
        Ack(ChannelOpenAck),
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregatePacketUpdateClient<L: LightClient> {
        // Will be threaded through to the update msg
        pub update_to: HeightOf<L::HostChain>,
        pub event_height: HeightOf<L::HostChain>,
        pub block_hash: H256,
        pub packet_event: PacketEvent,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub enum PacketEvent {
        Send(SendPacket),
        Recv(RecvPacket),
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregateFetchCounterpartyStateProof<L: LightClient> {
        pub counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
        pub fetch: FetchStateProof<L::Counterparty>,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregateUpdateClientFromClientId<L: LightClient> {
        pub client_id: L::ClientId,
        pub counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregateUpdateClient<L: LightClient> {
        pub update_to: HeightOf<L::HostChain>,
        pub client_id: L::ClientId,
        pub counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregateWaitForTrustedHeight<L: LightClient> {
        pub wait_for: HeightOf<L::HostChain>,
        pub client_id: L::ClientId,
        pub counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregateUpdateClientWithCounterpartyChainId<L: LightClient> {
        pub update_to: HeightOf<L::HostChain>,
        pub client_id: L::ClientId,
        pub counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
        pub counterparty_chain_id: ChainIdOf<L::Counterparty>,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregateMsgUpdateClient<L: LightClient> {
        pub update_to: HeightOf<L::HostChain>,
        pub client_id: L::ClientId,
        pub counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
        pub counterparty_chain_id: ChainIdOf<L::Counterparty>,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregateCreateClient<L: LightClient> {
        pub config: L::Config,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct ConsensusStateProofAtLatestHeight<L: LightClient> {
        pub client_id: L::ClientId,
        pub at: HeightOf<ChainOf<L>>,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct LightClientSpecificAggregate<L: LightClient>(pub L::Aggregate);

    /// Messages that will be re-queued after an update.
    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub enum AggregateMsgAfterUpdate<L: LightClient> {
        ConnectionOpenTry(AggregateConnectionOpenTry<L>),
        ConnectionOpenAck(AggregateConnectionOpenAck<L>),
        ConnectionOpenConfirm(AggregateConnectionOpenConfirm<L>),

        ChannelOpenTry(AggregateChannelOpenTry<L>),
        ChannelOpenAck(AggregateChannelOpenAck<L>),
        ChannelOpenConfirm(AggregateChannelOpenConfirm<L>),

        RecvPacket(AggregateRecvPacket<L>),
        AckPacket(AggregateAckPacket<L>),
    }
}

pub trait AnyLightClient {
    type Inner<L: LightClient>: Debug
        + Display
        + Clone
        + PartialEq
        + Serialize
        + for<'de> Deserialize<'de>;
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[serde(bound(serialize = "", deserialize = ""))]
#[allow(clippy::large_enum_variant)]
pub enum AnyLightClientIdentified<T: AnyLightClient> {
    // The 08-wasm client tracking the state of Evm<Mainnet>.
    #[display(fmt = "EthereumMainnet({}, {})", "_0.chain_id", "_0.data")]
    EthereumMainnet(Identified<EthereumMainnet, InnerOf<T, EthereumMainnet>>),
    // The 08-wasm client tracking the state of Evm<Minimal>.
    #[display(fmt = "EthereumMinimal({}, {})", "_0.chain_id", "_0.data")]
    EthereumMinimal(Identified<EthereumMinimal, InnerOf<T, EthereumMinimal>>),
    // The solidity client on Evm<Mainnet> tracking the state of Union.
    #[display(fmt = "CometblsMainnet({}, {})", "_0.chain_id", "_0.data")]
    CometblsMainnet(Identified<CometblsMainnet, InnerOf<T, CometblsMainnet>>),
    // The solidity client on Evm<Minimal> tracking the state of Union.
    #[display(fmt = "CometblsMinimal({}, {})", "_0.chain_id", "_0.data")]
    CometblsMinimal(Identified<CometblsMinimal, InnerOf<T, CometblsMinimal>>),
}

impl<T: AnyLightClient> From<Identified<EthereumMainnet, InnerOf<T, EthereumMainnet>>>
    for AnyLightClientIdentified<T>
{
    fn from(v: Identified<EthereumMainnet, InnerOf<T, EthereumMainnet>>) -> Self {
        Self::EthereumMainnet(v)
    }
}

impl<T: AnyLightClient> TryFrom<AnyLightClientIdentified<T>>
    for Identified<EthereumMainnet, InnerOf<T, EthereumMainnet>>
{
    type Error = AnyLightClientIdentified<T>;
    fn try_from(v: AnyLightClientIdentified<T>) -> Result<Self, Self::Error> {
        if let AnyLightClientIdentified::EthereumMainnet(v) = v {
            Ok(v)
        } else {
            Err(v)
        }
    }
}

impl<T: AnyLightClient> From<Identified<EthereumMinimal, InnerOf<T, EthereumMinimal>>>
    for AnyLightClientIdentified<T>
{
    fn from(v: Identified<EthereumMinimal, InnerOf<T, EthereumMinimal>>) -> Self {
        Self::EthereumMinimal(v)
    }
}

impl<T: AnyLightClient> TryFrom<AnyLightClientIdentified<T>>
    for Identified<EthereumMinimal, InnerOf<T, EthereumMinimal>>
{
    type Error = AnyLightClientIdentified<T>;
    fn try_from(v: AnyLightClientIdentified<T>) -> Result<Self, Self::Error> {
        if let AnyLightClientIdentified::EthereumMinimal(v) = v {
            Ok(v)
        } else {
            Err(v)
        }
    }
}

impl<T: AnyLightClient> From<Identified<CometblsMainnet, InnerOf<T, CometblsMainnet>>>
    for AnyLightClientIdentified<T>
{
    fn from(v: Identified<CometblsMainnet, InnerOf<T, CometblsMainnet>>) -> Self {
        Self::CometblsMainnet(v)
    }
}

impl<T: AnyLightClient> TryFrom<AnyLightClientIdentified<T>>
    for Identified<CometblsMainnet, InnerOf<T, CometblsMainnet>>
{
    type Error = AnyLightClientIdentified<T>;
    fn try_from(v: AnyLightClientIdentified<T>) -> Result<Self, Self::Error> {
        if let AnyLightClientIdentified::CometblsMainnet(v) = v {
            Ok(v)
        } else {
            Err(v)
        }
    }
}

impl<T: AnyLightClient> From<Identified<CometblsMinimal, InnerOf<T, CometblsMinimal>>>
    for AnyLightClientIdentified<T>
{
    fn from(v: Identified<CometblsMinimal, InnerOf<T, CometblsMinimal>>) -> Self {
        Self::CometblsMinimal(v)
    }
}

impl<T: AnyLightClient> TryFrom<AnyLightClientIdentified<T>>
    for Identified<CometblsMinimal, InnerOf<T, CometblsMinimal>>
{
    type Error = AnyLightClientIdentified<T>;
    fn try_from(v: AnyLightClientIdentified<T>) -> Result<Self, Self::Error> {
        if let AnyLightClientIdentified::CometblsMinimal(v) = v {
            Ok(v)
        } else {
            Err(v)
        }
    }
}

pub type AnyDataIdentified = AnyLightClientIdentified<AnyData>;

macro_rules! enum_variants_conversions {
    (
        $(#[$meta:meta])*
        pub enum $Enum:ident {
            $(
                $(#[$inner_meta:meta])*
                $Variant:ident($Inner:ty),
            )+
        }
    ) => {
        $(#[$meta])*
        pub enum $Enum {
            $(
                $(#[$inner_meta])*
                $Variant($Inner),
            )+
        }

        $(
            impl From<$Inner> for $Enum {
                fn from(inner: $Inner) -> Self {
                    Self::$Variant(inner)
                }
            }

            impl TryFrom<$Enum> for $Inner {
                type Error = $Enum;

                fn try_from(value: $Enum) -> Result<Self, Self::Error> {
                    match value {
                        $Enum::$Variant(inner) => Ok(inner),
                        _ => Err(value),
                    }
                }
            }
        )+
    };
}

pub(crate) use enum_variants_conversions;

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    // use hex_literal::hex;

    // use super::*;
    // use crate::{chain::cosmos::EthereumConfig, msg::CreateClientData};

    use std::{collections::VecDeque, fmt::Debug, marker::PhantomData};

    use hex_literal::hex;
    use serde::{de::DeserializeOwned, Serialize};
    use unionlabs::{
        ethereum::{Address, H256, U256},
        events::{ConnectionOpenAck, ConnectionOpenTry},
        ibc::core::{
            channel::{
                self, channel::Channel, msg_channel_open_init::MsgChannelOpenInit, order::Order,
            },
            commitment::merkle_prefix::MerklePrefix,
            connection::{
                self, msg_connection_open_init::MsgConnectionOpenInit,
                msg_connection_open_try::MsgConnectionOpenTry, version::Version,
            },
        },
        EmptyString,
    };

    use super::LcMsg;
    use crate::{
        chain::{
            evm::{CometblsConfig, CometblsMinimal},
            union::{EthereumConfig, EthereumMinimal},
            QueryHeight,
        },
        msg::{
            aggregate::{Aggregate, AggregateCreateClient, AnyAggregate},
            data::Data,
            event,
            event::{Event, IbcEvent},
            fetch,
            fetch::{
                AnyFetch, Fetch, FetchSelfClientState, FetchSelfConsensusState,
                FetchTrustedClientState,
            },
            msg,
            msg::{
                Msg, MsgChannelOpenInitData, MsgConnectionOpenInitData, MsgConnectionOpenTryData,
            },
            AggregateReceiver, AnyLcMsg, AnyMsg, Identified, RelayerMsg,
        },
        DELAY_PERIOD,
    };

    macro_rules! parse {
        ($expr:expr) => {
            $expr.parse().unwrap()
        };
    }

    #[test]
    fn msg_serde() {
        let union_chain_id: String = parse!("union-devnet-1");
        let eth_chain_id: U256 = parse!("32382");

        print_json(msg::<EthereumMinimal>(
            union_chain_id.clone(),
            MsgConnectionOpenInitData {
                msg: MsgConnectionOpenInit {
                    client_id: parse!("08-wasm-2"),
                    counterparty: connection::counterparty::Counterparty {
                        client_id: parse!("cometbls-0"),
                        connection_id: EmptyString,
                        prefix: MerklePrefix {
                            key_prefix: b"ibc".to_vec(),
                        },
                    },
                    version: Version {
                        identifier: "1".into(),
                        features: [Order::Ordered, Order::Unordered].into_iter().collect(),
                    },
                    delay_period: DELAY_PERIOD,
                },
            },
        ));

        print_json(msg::<EthereumMinimal>(
            union_chain_id.clone(),
            MsgChannelOpenInitData {
                msg: MsgChannelOpenInit {
                    port_id: "ping-pong".to_string(),
                    channel: Channel {
                        state: channel::state::State::Init,
                        ordering: channel::order::Order::Unordered,
                        counterparty: channel::counterparty::Counterparty {
                            port_id: "WASM_PORT_ID".to_string(),
                            channel_id: EmptyString.to_string(),
                        },
                        connection_hops: vec![parse!("connection-8")],
                        version: "ucs00-pingpong-1".to_string(),
                    },
                },
                __marker: PhantomData,
            },
        ));

        print_json(msg::<CometblsMinimal>(
            eth_chain_id,
            MsgChannelOpenInitData {
                msg: MsgChannelOpenInit {
                    port_id: "transfer".to_string(),
                    channel: Channel {
                        state: channel::state::State::Init,
                        ordering: channel::order::Order::Ordered,
                        counterparty: channel::counterparty::Counterparty {
                            port_id: "transfer".to_string(),
                            channel_id: EmptyString.to_string(),
                        },
                        connection_hops: vec![parse!("connection-8")],
                        version: "ucs001-pingpong".to_string(),
                    },
                },
                __marker: PhantomData,
            },
        ));

        print_json(msg::<CometblsMinimal>(
            eth_chain_id,
            MsgConnectionOpenInitData {
                msg: MsgConnectionOpenInit {
                    client_id: parse!("cometbls-0"),
                    counterparty: connection::counterparty::Counterparty {
                        client_id: parse!("08-wasm-0"),
                        connection_id: EmptyString,
                        prefix: MerklePrefix {
                            key_prefix: b"ibc".to_vec(),
                        },
                    },
                    version: Version {
                        identifier: "1".into(),
                        features: [Order::Ordered, Order::Unordered].into_iter().collect(),
                    },
                    delay_period: DELAY_PERIOD,
                },
            },
        ));

        print_json(event::<CometblsMinimal>(
            eth_chain_id,
            IbcEvent {
                block_hash: H256([0; 32]),
                height: parse!("0-2941"),
                event: unionlabs::events::IbcEvent::ConnectionOpenTry(ConnectionOpenTry {
                    connection_id: parse!("connection-0"),
                    client_id: parse!("cometbls-0"),
                    counterparty_client_id: parse!("08-wasm-1"),
                    counterparty_connection_id: parse!("connection-14"),
                }),
            },
        ));

        print_json(RelayerMsg::Timeout {
            timeout_timestamp: u64::MAX,
            msg: Box::new(event::<CometblsMinimal>(
                eth_chain_id,
                crate::msg::event::Command::UpdateClient {
                    client_id: parse!("cometbls-0"),
                    counterparty_client_id: parse!("08-wasm-0"),
                },
            )),
        });

        print_json(RelayerMsg::Timeout {
            timeout_timestamp: u64::MAX,
            msg: Box::new(event::<EthereumMinimal>(
                union_chain_id.clone(),
                crate::msg::event::Command::UpdateClient {
                    client_id: parse!("08-wasm-0"),
                    counterparty_client_id: parse!("cometbls-0"),
                },
            )),
        });

        println!("\ncreate client msgs\n");

        print_json(RelayerMsg::Sequence(
            [
                RelayerMsg::Aggregate {
                    queue: [
                        fetch::<EthereumMinimal>(
                            union_chain_id.clone(),
                            FetchSelfClientState {
                                at: QueryHeight::Latest,
                            },
                        ),
                        fetch::<EthereumMinimal>(
                            union_chain_id.clone(),
                            FetchSelfConsensusState {
                                at: QueryHeight::Latest,
                            },
                        )
                    ]
                    .into(),
                    data: [].into_iter().collect(),
                    receiver: AggregateReceiver::CometblsMinimal(Identified {
                        chain_id: eth_chain_id,
                        data: Aggregate::CreateClient(AggregateCreateClient {
                            config: CometblsConfig {
                                client_type: "cometbls".to_string(),
                                cometbls_client_address: Address(hex!(
                                    "83428c7db9815f482a39a1715684dcf755021997"
                                )),
                            },
                        }),
                    }),
                },
                RelayerMsg::Aggregate {
                    queue: [
                        fetch::<CometblsMinimal>(
                            eth_chain_id,
                            FetchSelfClientState {
                                at: QueryHeight::Latest,
                            },
                        ),
                        fetch::<CometblsMinimal>(
                            eth_chain_id,
                            FetchSelfConsensusState {
                                at: QueryHeight::Latest,
                            },
                        )
                    ]
                    .into(),
                    data: [].into_iter().collect(),
                    receiver: AggregateReceiver::EthereumMinimal(Identified {
                        chain_id: union_chain_id.clone(),
                        data: Aggregate::CreateClient(AggregateCreateClient {
                            config: EthereumConfig {
                                code_id: H256(hex!(
                                    "78266014ea77f3b785e45a33d1f8d3709444a076b3b38b2aeef265b39ad1e494"
                                )),
                            },
                        }),
                    }),
                },
            ]
            .into(),
        ));

        // print_json(RelayerMsg::Lc(AnyLcMsg::EthereumMinimal(LcMsg::Event(
        //     Identified {
        //         chain_id: union_chain_id.clone(),
        //         data: crate::msg::event::Event {
        //             block_hash: H256([0; 32]),
        //             height: parse!("1-1433"),
        //             event: IbcEvent::ConnectionOpenAck(ConnectionOpenAck {
        //                 connection_id: parse!("connection-5"),
        //                 client_id: parse!("08-wasm-0"),
        //                 counterparty_client_id: parse!("cometbls-0"),
        //                 counterparty_connection_id: parse!("connection-4"),
        //             }),
        //         },
        //     },
        // ))));
    }

    fn print_json<T: Serialize + DeserializeOwned + PartialEq + Debug>(msg: T) {
        let json = serde_json::to_string(&msg).unwrap();

        println!("{json}\n");

        let from_json = serde_json::from_str(&json).unwrap();

        assert_eq!(&msg, &from_json, "json roundtrip failed");
    }
}

macro_rules! identified {
    ($Ty:ident<$L:ty>) => {
        crate::msg::Identified<$L, $Ty<$L>>
    };
}

pub(crate) use identified;

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[serde(bound(serialize = "", deserialize = ""))]
#[allow(clippy::large_enum_variant)]
pub enum LcMsg<L: LightClient> {
    #[display(fmt = "Event({})", "_0")]
    Event(InnerOf<AnyEvent, L>),
    // data that has been read
    #[display(fmt = "Data({})", "_0")]
    Data(InnerOf<AnyData, L>),
    // read
    #[display(fmt = "Fetch({})", "_0")]
    Fetch(InnerOf<AnyFetch, L>),
    // write
    #[display(fmt = "Msg({})", "_0")]
    Msg(InnerOf<AnyMsg, L>),
    #[display(fmt = "Wait({})", "_0")]
    Wait(InnerOf<AnyWait, L>),
    // REVIEW: Does this make sense as a top-level message?
    #[display(fmt = "Aggregate({})", "_0")]
    Aggregate(InnerOf<AnyAggregate, L>),
}

pub type InnerOf<T, L> = <T as AnyLightClient>::Inner<L>;

pub enum AnyLcMsg {}

impl AnyLightClient for AnyLcMsg {
    type Inner<L: LightClient> = LcMsg<L>;
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(
    serialize = "Data: ::serde::Serialize",
    deserialize = "Data: for<'d> Deserialize<'d>"
))]
// TODO: `Data: AnyLightClient`
// prerequisites: derive macro for AnyLightClient
pub struct Identified<L: LightClient, Data: Debug + Clone + PartialEq> {
    pub chain_id: ChainIdOf<L>,
    pub data: Data,
}

impl<L: LightClient, Data: Debug + Clone + PartialEq> Identified<L, Data> {
    pub fn new(chain_id: ChainIdOf<L>, data: Data) -> Self {
        Self { chain_id, data }
    }
}

pub trait DoAggregate<L>: Sized + Debug + Clone + PartialEq
where
    L: LightClient,
{
    fn do_aggregate(
        _: Identified<L, Self>,
        _: VecDeque<AnyLightClientIdentified<AnyData>>,
    ) -> Vec<RelayerMsg>;
}

// helper fns

pub fn retry(count: u8, t: impl Into<RelayerMsg>) -> RelayerMsg {
    RelayerMsg::Retry(count, Box::new(t.into()))
}

pub fn seq(ts: impl IntoIterator<Item = RelayerMsg>) -> RelayerMsg {
    RelayerMsg::Sequence(ts.into_iter().collect())
}

pub fn defer(timestamp: u64) -> RelayerMsg {
    RelayerMsg::DeferUntil {
        point: DeferPoint::Absolute,
        seconds: timestamp,
    }
}

pub fn defer_relative(seconds: u64) -> RelayerMsg {
    RelayerMsg::DeferUntil {
        point: DeferPoint::Relative,
        seconds,
    }
}

pub fn fetch<L: LightClient>(chain_id: ChainIdOf<L>, t: impl Into<Fetch<L>>) -> RelayerMsg
where
    AnyLightClientIdentified<AnyLcMsg>: From<identified!(LcMsg<L>)>,
{
    RelayerMsg::Lc(AnyLightClientIdentified::from(Identified::new(
        chain_id,
        LcMsg::Fetch(t.into()),
    )))
}

pub fn msg<L: LightClient>(chain_id: ChainIdOf<L>, t: impl Into<Msg<L>>) -> RelayerMsg
where
    AnyLightClientIdentified<AnyLcMsg>: From<identified!(LcMsg<L>)>,
{
    RelayerMsg::Lc(AnyLightClientIdentified::from(Identified::new(
        chain_id,
        LcMsg::Msg(t.into()),
    )))
}

pub fn data<L: LightClient>(chain_id: ChainIdOf<L>, t: impl Into<Data<L>>) -> RelayerMsg
where
    AnyLightClientIdentified<AnyLcMsg>: From<identified!(LcMsg<L>)>,
{
    RelayerMsg::Lc(AnyLightClientIdentified::from(Identified::new(
        chain_id,
        LcMsg::Data(t.into()),
    )))
}

pub fn wait<L: LightClient>(chain_id: ChainIdOf<L>, t: impl Into<Wait<L>>) -> RelayerMsg
where
    AnyLightClientIdentified<AnyLcMsg>: From<identified!(LcMsg<L>)>,
{
    RelayerMsg::Lc(AnyLightClientIdentified::from(Identified::new(
        chain_id,
        LcMsg::Wait(t.into()),
    )))
}

pub fn event<L: LightClient>(chain_id: ChainIdOf<L>, t: impl Into<Event<L>>) -> RelayerMsg
where
    AnyLightClientIdentified<AnyLcMsg>: From<identified!(LcMsg<L>)>,
{
    RelayerMsg::Lc(AnyLightClientIdentified::from(Identified::new(
        chain_id,
        LcMsg::Event(t.into()),
    )))
}
