use std::num::NonZeroU64;

use enumorph::Enumorph;
use macros::model;
use serde::de::DeserializeOwned;
use serde_json::Value;
use subset_of::SubsetOf;
use tracing::info;
use unionlabs::{
    bytes::Bytes,
    hash::H256,
    ibc::core::{
        channel::{
            msg_acknowledgement::MsgAcknowledgement, msg_channel_open_ack::MsgChannelOpenAck,
            msg_channel_open_confirm::MsgChannelOpenConfirm,
            msg_channel_open_init::MsgChannelOpenInit, msg_channel_open_try::MsgChannelOpenTry,
            msg_recv_packet::MsgRecvPacket, msg_timeout::MsgTimeout, order::Order,
        },
        client::{
            height::Height, msg_create_client::MsgCreateClient, msg_update_client::MsgUpdateClient,
        },
        connection::{
            connection_end::ConnectionEnd, msg_connection_open_ack::MsgConnectionOpenAck,
            msg_connection_open_confirm::MsgConnectionOpenConfirm,
            msg_connection_open_init::MsgConnectionOpenInit,
            msg_connection_open_try::MsgConnectionOpenTry,
        },
    },
    ics24::{ClientConsensusStatePath, ClientStatePath, IbcPath, Path},
    id::{ChannelId, ClientId, ConnectionId, PortId},
    traits::Member,
};
use voyager_core::IbcVersionId;

use crate::{
    core::{ChainId, ClientInfo, ClientStateMeta, ClientType, ConsensusStateMeta},
    PluginMessage,
};

#[model]
#[derive(Enumorph, SubsetOf)]
#[allow(clippy::large_enum_variant)]
pub enum Data {
    IbcEvent(ChainEvent),
    IbcDatagram(IbcDatagram),

    IdentifiedIbcDatagram(WithChainId<IbcDatagram>),
    IdentifiedIbcDatagramBatch(WithChainId<Vec<IbcDatagram>>),

    OrderedHeaders(OrderedHeaders),
    OrderedMsgUpdateClients(OrderedMsgUpdateClients),

    Plugin(PluginMessage),
}

impl Data {
    #[allow(clippy::result_large_err)]
    pub fn as_plugin<T: DeserializeOwned>(self, plugin_name: impl AsRef<str>) -> Result<T, Self> {
        match self {
            Self::Plugin(plugin_message) => {
                plugin_message.downcast(plugin_name).map_err(Self::Plugin)
            }
            this => Err(this),
        }
    }
}

#[model]
pub struct ChainEvent {
    /// The chain where this event was emitted.
    pub chain_id: ChainId,
    /// The underlying client of this event, on [`Self::chain_id`].
    pub client_info: ClientInfo,
    /// The chain on the other end of this IBC event.
    pub counterparty_chain_id: ChainId,
    pub tx_hash: H256,
    /// The "provable height" of the event. This is the minimum height at which
    /// the effect of the IBC action that caused this event is provable in
    /// the state root of the chain identified by [`Self::chain_id`].
    pub provable_height: Height,

    pub ibc_version_id: IbcVersionId<'static>,
    /// The full IBC event, encoded as JSON value. This is really [`IbcSpec::Event`],
    /// and will be interpreted based on the implementation defined by [`Self::ibc_version_id`].
    pub event: Value,
}

#[model]
pub struct IbcDatagram {
    pub ibc_version_id: IbcVersionId<'static>,
    /// The IBC datagram, encoded as JSON value. This is really [`IbcSpec::Datagram`],
    /// and will be interpreted based on the implementation defined by [`Self::ibc_version_id`].
    pub event: Value,
}

#[model]
pub struct UnfinalizedTrustedClientState {
    pub height: Height,
    pub client_state: ClientStateMeta,
}

#[model]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct IbcState<P: IbcPath> {
    pub chain_id: ChainId,
    pub path: P,
    /// The height that the state was read at.
    pub height: Height,
    pub state: P::Value,
}

#[model]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct IbcProof<P: IbcPath> {
    pub path: P,
    pub height: Height,
    pub proof: Bytes,
}

#[model]
pub struct RawIbcProof {
    pub path: Path,
    pub height: Height,
    /// The raw proof, encoded as JSON, which will be encoded by the relevant
    /// client module.
    pub proof: Value,
}

#[model]
pub struct DecodedHeaderMeta {
    /// The new trusted height that the header provides a consensus update to.
    pub height: Height,
}

#[model]
pub struct OrderedHeaders {
    pub headers: Vec<(DecodedHeaderMeta, Value)>,
}

#[model]
pub struct OrderedMsgUpdateClients {
    pub updates: Vec<(DecodedHeaderMeta, MsgUpdateClient)>,
}

#[model]
pub struct WithChainId<T> {
    pub chain_id: ChainId,
    pub message: T,
}
