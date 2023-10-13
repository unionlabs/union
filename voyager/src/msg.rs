#![allow(clippy::type_complexity)]

use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
};

use chain_utils::{Chain, ClientState};
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use serde::{Deserialize, Serialize};

use crate::{
    chain::{
        evm::{CometblsMainnet, CometblsMinimal},
        proof::{IbcPath, StateProof},
        union::{EthereumMainnet, EthereumMinimal},
        LightClient,
    },
    msg::{
        aggregate::{Aggregate, AnyAggregate},
        data::{AnyData, Data},
        event::AnyEvent,
        fetch::AnyFetch,
        msg::AnyMsg,
        wait::AnyWait,
    },
};

pub mod data;
pub mod event;
pub mod fetch;
#[allow(clippy::module_inception)] // fight me clippy
pub mod msg;
pub mod wait;

pub type ChainIdOf<L> =
    <<<L as LightClient>::HostChain as Chain>::SelfClientState as ClientState>::ChainId;

pub type StateProofOf<T, L> = StateProof<
    <T as IbcPath<
        <L as LightClient>::HostChain,
        <<L as LightClient>::Counterparty as LightClient>::HostChain,
    >>::Output,
>;

pub trait IntoRelayerMsg {
    fn into_relayer_msg(self) -> RelayerMsg;
}

pub trait TryFromRelayerMsg: Sized {
    fn try_from_relayer_msg(msg: RelayerMsg) -> Result<Self, RelayerMsg>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum RelayerMsg {
    Lc(AnyLcMsg),
    DeferUntil {
        timestamp: u64,
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
        data: VecDeque<AggregateData>,
        /// The message that will utilize the aggregated data.
        receiver: AggregateReceiver,
    },
}

impl std::fmt::Display for RelayerMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelayerMsg::Lc(lc) => write!(f, "Lc({lc})"),
            RelayerMsg::DeferUntil { timestamp } => write!(f, "DeferUntil({timestamp})"),
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

                write!(f, "Aggregate([{queue}], [{data}] -> {receiver})")
            }
        }
    }
}

enum_variants_conversions! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub enum AggregateData {
        // The 08-wasm client tracking the state of Evm<Mainnet>.
        EthereumMainnet(identified!(Data<EthereumMainnet>)),
        // The 08-wasm client tracking the state of Evm<Minimal>.
        EthereumMinimal(identified!(Data<EthereumMinimal>)),
        // The solidity client on Evm<Mainnet> tracking the state of Union.
        CometblsMainnet(identified!(Data<CometblsMainnet>)),
        // The solidity client on Evm<Minimal> tracking the state of Union.
        CometblsMinimal(identified!(Data<CometblsMinimal>)),
    }
}

impl Display for AggregateData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AggregateData::EthereumMainnet(data) => {
                write!(f, "Data::EthereumMainnet({}, {})", data.chain_id, data.data)
            }
            AggregateData::EthereumMinimal(data) => {
                write!(f, "Data::EthereumMinimal({}, {})", data.chain_id, data.data)
            }
            AggregateData::CometblsMainnet(data) => {
                write!(f, "Data::CometblsMainnet({}, {})", data.chain_id, data.data)
            }
            AggregateData::CometblsMinimal(data) => {
                write!(f, "Data::CometblsMinimal({}, {})", data.chain_id, data.data)
            }
        }
    }
}

impl TryFrom<RelayerMsg> for AggregateData {
    type Error = RelayerMsg;

    fn try_from(value: RelayerMsg) -> Result<Self, Self::Error> {
        match value {
            RelayerMsg::Lc(AnyLcMsg::EthereumMainnet(LcMsg::Data(data))) => {
                Ok(AggregateData::EthereumMainnet(data))
            }
            RelayerMsg::Lc(AnyLcMsg::EthereumMinimal(LcMsg::Data(data))) => {
                Ok(AggregateData::EthereumMinimal(data))
            }
            RelayerMsg::Lc(AnyLcMsg::CometblsMainnet(LcMsg::Data(data))) => {
                Ok(AggregateData::CometblsMainnet(data))
            }
            RelayerMsg::Lc(AnyLcMsg::CometblsMinimal(LcMsg::Data(data))) => {
                Ok(AggregateData::CometblsMinimal(data))
            }
            _ => Err(value),
        }
    }
}

enum_variants_conversions! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_more::Display)]
    // TODO: Rename this
    pub enum AggregateReceiver {
        // The 08-wasm client tracking the state of Evm<Mainnet>.
        #[display(fmt = "EthereumMainnet({}, {})", "_0.chain_id", "_0.data")]
        EthereumMainnet(identified!(Aggregate<EthereumMainnet>)),
        // The 08-wasm client tracking the state of Evm<Minimal>.
        #[display(fmt = "EthereumMinimal({}, {})", "_0.chain_id", "_0.data")]
        EthereumMinimal(identified!(Aggregate<EthereumMinimal>)),
        // The solidity client on Evm<Mainnet> tracking the state of Union.
        #[display(fmt = "CometblsMainnet({}, {})", "_0.chain_id", "_0.data")]
        CometblsMainnet(identified!(Aggregate<CometblsMainnet>)),
        // The solidity client on Evm<Minimal> tracking the state of Union.
        #[display(fmt = "CometblsMinimal({}, {})", "_0.chain_id", "_0.data")]
        CometblsMinimal(identified!(Aggregate<CometblsMinimal>)),
    }
}

impl TryFrom<RelayerMsg> for AnyLcMsg {
    type Error = RelayerMsg;

    fn try_from(value: RelayerMsg) -> Result<Self, Self::Error> {
        match value {
            RelayerMsg::Lc(ok) => Ok(ok),
            _ => Err(value),
        }
    }
}

impl From<AnyLcMsg> for RelayerMsg {
    fn from(value: AnyLcMsg) -> Self {
        Self::Lc(value)
    }
}

impl<L: LightClient> TryFrom<RelayerMsg> for LcMsg<L>
where
    AnyLcMsg: TryFrom<RelayerMsg, Error = RelayerMsg> + Into<RelayerMsg>,
    LcMsg<L>: TryFrom<AnyLcMsg, Error = AnyLcMsg> + Into<AnyLcMsg>,
{
    type Error = RelayerMsg;

    fn try_from(value: RelayerMsg) -> Result<Self, Self::Error> {
        LcMsg::<L>::try_from(AnyLcMsg::try_from(value)?).map_err(Into::into)
    }
}

impl<L: LightClient> From<LcMsg<L>> for RelayerMsg
where
    AnyLcMsg: From<LcMsg<L>>,
{
    fn from(value: LcMsg<L>) -> Self {
        RelayerMsg::Lc(AnyLcMsg::from(value))
    }
}

macro_rules! any_enum {
    (
        $(#[doc = $outer_doc:literal])*
        #[any = $Any:ident($AnyInner:ty)]
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
            type Inner<L: LightClient> = $AnyInner;
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
                    crate::msg::AnyLcMsg: TryFrom<crate::msg::RelayerMsg, Error = crate::msg::RelayerMsg> + Into<crate::msg::RelayerMsg>,
                    crate::msg::LcMsg<L>: TryFrom<crate::msg::AnyLcMsg, Error = crate::msg::AnyLcMsg> + Into<crate::msg::AnyLcMsg>,
                    crate::msg::Identified<L, $VariantInner>: TryFrom<crate::msg::LcMsg<L>, Error = crate::msg::LcMsg<L>> + Into<crate::msg::LcMsg<L>>,
                {
                    type Error = crate::msg::RelayerMsg;

                    fn try_into(self) -> Result<crate::msg::Identified<L, $VariantInner>, crate::msg::RelayerMsg> {
                        crate::msg::AnyLcMsg::try_from(self)
                            .and_then(|x| <crate::msg::LcMsg<L>>::try_from(x).map_err(Into::into))
                            .and_then(|x| {
                                <crate::msg::Identified<L, $VariantInner>>::try_from(x)
                                    .map_err(Into::<crate::msg::LcMsg<L>>::into)
                                    .map_err(Into::<crate::msg::AnyLcMsg>::into)
                                    .map_err(Into::<crate::msg::RelayerMsg>::into)
                            })
                    }
                }

                impl<L: LightClient> TryFrom<crate::msg::LcMsg<L>> for crate::msg::Identified<L, $VariantInner> {
                    type Error = crate::msg::LcMsg<L>;

                    fn try_from(value: crate::msg::LcMsg<L>) -> Result<Self, crate::msg::LcMsg<L>> {
                        match value {
                            crate::msg::LcMsg::$Enum(crate::msg::Identified {
                                chain_id,
                                data: $Enum::$Variant(data),
                            }) => Ok(crate::msg::Identified { chain_id, data }),
                            _ => Err(value),
                        }
                    }
                }

                impl<L: LightClient> TryFrom<crate::msg::AnyLcMsg> for crate::msg::Identified<L, $VariantInner>
                where
                    crate::msg::LcMsg<L>: TryFrom<crate::msg::AnyLcMsg, Error = crate::msg::AnyLcMsg> + Into<crate::msg::AnyLcMsg>,
                    Self: TryFrom<crate::msg::LcMsg<L>, Error = crate::msg::LcMsg<L>> + Into<crate::msg::LcMsg<L>>,
                {
                    type Error = crate::msg::AnyLcMsg;

                    fn try_from(value: crate::msg::AnyLcMsg) -> Result<Self, crate::msg::AnyLcMsg> {
                        crate::msg::LcMsg::<L>::try_from(value).and_then(|x| Self::try_from(x).map_err(Into::into))
                    }
                }
            )?
        )+
    };
}

pub(crate) use any_enum;

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
        chain::{proof::AcknowledgementPath, ChainOf, HeightOf, LightClient},
        msg::{fetch::FetchStateProof, identified},
    };

    any_enum! {
        /// Aggregate data, using data from [`AggregateData`]
        #[any = AnyAggregate(identified!(Aggregate<L>))]
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
                Aggregate::PacketUpdateClient(_) => write!(f, "PacketUpdateClient"),
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
                Aggregate::AggregateMsgAfterUpdate(_) => write!(f, "AggregateMsgAfterUpdate"),
                Aggregate::LightClientSpecific(agg) => write!(f, "LightClientSpecific({})", agg.0),
            }
        }
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregateConnectionOpenTry<L: LightClient> {
        pub event_height: HeightOf<L::HostChain>,
        pub event: ConnectionOpenInit<L::ClientId, <L::Counterparty as LightClient>::ClientId>,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregateConnectionOpenAck<L: LightClient> {
        pub event_height: HeightOf<L::HostChain>,
        pub event: ConnectionOpenTry<L::ClientId, <L::Counterparty as LightClient>::ClientId>,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregateConnectionOpenConfirm<L: LightClient> {
        pub event_height: HeightOf<L::HostChain>,
        pub event: ConnectionOpenAck<L::ClientId, <L::Counterparty as LightClient>::ClientId>,
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
        pub counterparty_client_id: <L::Counterparty as LightClient>::ClientId,
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
        pub counterparty_client_id: <L::Counterparty as LightClient>::ClientId,
        pub fetch: FetchStateProof<L::Counterparty>,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregateUpdateClientFromClientId<L: LightClient> {
        pub client_id: L::ClientId,
        pub counterparty_client_id: <L::Counterparty as LightClient>::ClientId,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregateUpdateClient<L: LightClient> {
        pub update_to: HeightOf<L::HostChain>,
        pub client_id: L::ClientId,
        pub counterparty_client_id: <L::Counterparty as LightClient>::ClientId,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregateWaitForTrustedHeight<L: LightClient> {
        pub wait_for: HeightOf<L::HostChain>,
        pub client_id: L::ClientId,
        pub counterparty_client_id: <L::Counterparty as LightClient>::ClientId,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregateUpdateClientWithCounterpartyChainId<L: LightClient> {
        pub update_to: HeightOf<L::HostChain>,
        pub client_id: L::ClientId,
        pub counterparty_client_id: <L::Counterparty as LightClient>::ClientId,
        pub counterparty_chain_id: ChainIdOf<L::Counterparty>,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""))]
    pub struct AggregateMsgUpdateClient<L: LightClient> {
        pub update_to: HeightOf<L::HostChain>,
        pub client_id: L::ClientId,
        pub counterparty_client_id: <L::Counterparty as LightClient>::ClientId,
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

impl<L: LightClient, T> TryFrom<AggregateData> for Identified<L, T>
where
    T: TryFrom<Data<L>, Error = Data<L>> + Into<Data<L>> + Debug + Clone + PartialEq,
    identified!(Data<L>): TryFrom<AggregateData, Error = AggregateData> + Into<AggregateData>,
{
    type Error = AggregateData;

    fn try_from(value: AggregateData) -> Result<Self, Self::Error> {
        let Identified { chain_id, data } = <identified!(Data<L>)>::try_from(value)?;

        match T::try_from(data) {
            Ok(t) => Ok(Identified { chain_id, data: t }),
            Err(data) => Err(Identified { chain_id, data }.into()),
        }
    }
}

pub trait AnyLightClient {
    type Inner<L: LightClient>: Debug + Clone + PartialEq + Serialize + for<'de> Deserialize<'de>;
}

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
        events::{ConnectionOpenAck, ConnectionOpenTry, IbcEvent},
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
        chain::{evm::CometblsConfig, union::EthereumConfig, QueryHeight},
        msg::{
            aggregate::{Aggregate, AggregateCreateClient, AnyAggregate},
            data::Data,
            event::Event,
            fetch::{
                AnyFetch, Fetch, FetchSelfClientState, FetchSelfConsensusState,
                FetchTrustedClientState,
            },
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

        print_json(RelayerMsg::Lc(AnyLcMsg::EthereumMinimal(LcMsg::Msg(
            Identified {
                chain_id: union_chain_id.clone(),
                data: Msg::ConnectionOpenInit(MsgConnectionOpenInitData {
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
                }),
            },
        ))));

        print_json(RelayerMsg::Lc(AnyLcMsg::EthereumMinimal(LcMsg::Msg(
            Identified {
                chain_id: union_chain_id.clone(),
                data: Msg::ChannelOpenInit(MsgChannelOpenInitData {
                    msg: MsgChannelOpenInit {
                                port_id: "ping-pong".to_string(),
                        channel: Channel {
                            state: channel::state::State::Init,
                            ordering: channel::order::Order::Unordered,
                            counterparty: channel::counterparty::Counterparty {
                        port_id: "wasm.union12zp282rjmvh0jkqprmx2y7hphqlz45za6uxfekp0mz9vfxp4c0ts2gp3ss"
                                .to_string(),
                                channel_id: EmptyString.to_string(),
                            },
                            connection_hops: vec![parse!("connection-8")],
                            version: "ucs00-pingpong-1".to_string(),
                        },
                    },
                    __marker: PhantomData,
                }),
            },
        ))));

        print_json(RelayerMsg::Lc(AnyLcMsg::CometblsMinimal(LcMsg::Msg(
            Identified {
                chain_id: eth_chain_id,
                data: Msg::ChannelOpenInit(MsgChannelOpenInitData {
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
                }),
            },
        ))));

        print_json(RelayerMsg::Lc(AnyLcMsg::CometblsMinimal(LcMsg::Msg(
            Identified {
                chain_id: eth_chain_id,
                data: Msg::ConnectionOpenInit(MsgConnectionOpenInitData {
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
                }),
            },
        ))));

        print_json(RelayerMsg::Lc(AnyLcMsg::CometblsMinimal(LcMsg::Event(
            Identified {
                chain_id: eth_chain_id,
                data: crate::msg::event::Event::Ibc(crate::msg::event::IbcEvent {
                    block_hash: H256([0; 32]),
                    height: parse!("0-2941"),
                    event: IbcEvent::ConnectionOpenTry(ConnectionOpenTry {
                        connection_id: parse!("connection-0"),
                        client_id: parse!("cometbls-0"),
                        counterparty_client_id: parse!("08-wasm-1"),
                        counterparty_connection_id: parse!("connection-14"),
                    }),
                }),
            },
        ))));

        print_json(RelayerMsg::Timeout {
            timeout_timestamp: 1,
            msg: Box::new(RelayerMsg::Lc(AnyLcMsg::CometblsMinimal(LcMsg::Event(
                Identified {
                    chain_id: eth_chain_id,
                    data: crate::msg::event::Event::Command(
                        crate::msg::event::Command::UpdateClient {
                            client_id: parse!("cometbls-0"),
                            counterparty_client_id: parse!("08-wasm-0"),
                        },
                    ),
                },
            )))),
        });

        println!("\ncreate client msgs\n");

        print_json(RelayerMsg::Sequence(
            [
                RelayerMsg::Aggregate {
                    queue: [
                        RelayerMsg::Lc(AnyLcMsg::EthereumMinimal(LcMsg::Fetch(Identified {
                            chain_id: union_chain_id.clone(),
                            data: Fetch::SelfClientState(FetchSelfClientState {
                                at: QueryHeight::Latest,
                            }),
                        }))),
                        RelayerMsg::Lc(AnyLcMsg::EthereumMinimal(LcMsg::Fetch(Identified {
                            chain_id: union_chain_id.clone(),
                            data: Fetch::SelfConsensusState(FetchSelfConsensusState {
                                at: QueryHeight::Latest,
                            }),
                        }))),
                    ]
                    .into(),
                    data: [].into_iter().collect(),
                    receiver: AggregateReceiver::CometblsMinimal(Identified {
                        chain_id: eth_chain_id,
                        data: Aggregate::CreateClient(AggregateCreateClient {
                            config: CometblsConfig {
                                cometbls_client_address: Address(hex!(
                                    "83428c7db9815f482a39a1715684dcf755021997"
                                )),
                            },
                        }),
                    }),
                },
                RelayerMsg::Aggregate {
                    queue: [
                        RelayerMsg::Lc(AnyLcMsg::CometblsMinimal(LcMsg::Fetch(Identified {
                            chain_id: eth_chain_id,
                            data: Fetch::SelfClientState(FetchSelfClientState {
                                at: QueryHeight::Latest,
                            }),
                        }))),
                        RelayerMsg::Lc(AnyLcMsg::CometblsMinimal(LcMsg::Fetch(Identified {
                            chain_id: eth_chain_id,
                            data: Fetch::SelfConsensusState(FetchSelfConsensusState {
                                at: QueryHeight::Latest,
                            }),
                        }))),
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
    #[display(fmt = "Event({}, {})", "_0.chain_id", "_0.data")]
    Event(InnerOf<AnyEvent, L>),
    // data that has been read
    #[display(fmt = "Data({}, {})", "_0.chain_id", "_0.data")]
    Data(InnerOf<AnyData, L>),
    // read
    #[display(fmt = "Fetch({}, {})", "_0.chain_id", "_0.data")]
    Fetch(InnerOf<AnyFetch, L>),
    // write
    #[display(fmt = "Msg({}, {})", "_0.chain_id", "_0.data")]
    Msg(InnerOf<AnyMsg, L>),
    #[display(fmt = "Wait({}, {})", "_0.chain_id", "_0.data")]
    Wait(InnerOf<AnyWait, L>),
    // REVIEW: Does this make sense as a top-level message?
    #[display(fmt = "Aggregate({}, {})", "_0.chain_id", "_0.data")]
    Aggregate(InnerOf<AnyAggregate, L>),
}

type InnerOf<T, L> = <T as AnyLightClient>::Inner<L>;

enum_variants_conversions! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_more::Display)]
    pub enum AnyLcMsg {
        // The 08-wasm client tracking the state of Evm<Mainnet>.
        #[display(fmt = "EthereumMainnet({})", "_0")]
        EthereumMainnet(LcMsg<EthereumMainnet>),
        // The 08-wasm client tracking the state of Evm<Minimal>.
        #[display(fmt = "EthereumMinimal({})", "_0")]
        EthereumMinimal(LcMsg<EthereumMinimal>),
        // The solidity client on Evm<Mainnet> tracking the state of Union.
        #[display(fmt = "CometblsMainnet({})", "_0")]
        CometblsMainnet(LcMsg<CometblsMainnet>),
        // The solidity client on Evm<Minimal> tracking the state of Union.
        #[display(fmt = "CometblsMinimal({})", "_0")]
        CometblsMinimal(LcMsg<CometblsMinimal>),
    }
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(
    serialize = "Data: ::serde::Serialize",
    deserialize = "Data: for<'d> Deserialize<'d>"
))]
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
    fn do_aggregate(_: Identified<L, Self>, _: VecDeque<AggregateData>) -> Vec<RelayerMsg>;
}

#[test]
fn t() {
    use unionlabs::ethereum_consts_traits::{Mainnet, Minimal};

    use crate::chain::union::Validators;

    fn t<T: TryFrom<AggregateData> + Into<AggregateData>>() {}
    fn u<L: LightClient, T: TryFrom<Data<L>>>() {}

    t::<Identified<EthereumMinimal, Validators<Minimal>>>();
    u::<EthereumMinimal, Validators<Minimal>>();
    // u::<CometblsMinimal, Validators<Minimal>>();
}

fn t2() {
    enum T {
        U,
    }

    match T::U {
        T::U { .. } => {}
    }
}
