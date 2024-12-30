use enumorph::Enumorph;
use macros::model;
use serde::de::DeserializeOwned;
use serde_json::Value;
use subset_of::SubsetOf;
use unionlabs::{bytes::Bytes, hash::H256, ibc::core::client::height::Height, traits::Member};
use voyager_core::IbcSpecId;

use crate::{
    core::{ChainId, ClientInfo, ClientStateMeta, IbcSpec},
    into_value, PluginMessage, RawClientId,
};

#[model]
#[derive(Enumorph, SubsetOf)]
#[allow(clippy::large_enum_variant)]
pub enum Data {
    IbcEvent(ChainEvent),
    IbcDatagram(IbcDatagram),

    OrderedHeaders(OrderedHeaders),
    OrderedMsgUpdateClients(OrderedClientUpdates),

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

    pub ibc_spec_id: IbcSpecId,
    /// The full IBC event, encoded as JSON value. This is really [`IbcSpec::Event`],
    /// and will be interpreted based on the implementation defined by [`Self::ibc_spec_id`].
    pub event: Value,
}

impl ChainEvent {
    pub fn decode_event<V: IbcSpec>(&self) -> Option<Result<V::Event, serde_json::Error>> {
        if self.ibc_spec_id == V::ID {
            Some(serde_json::from_value(self.event.clone()))
        } else {
            None
        }
    }
}

#[model]
pub struct IbcDatagram {
    pub ibc_spec_id: IbcSpecId,
    /// The IBC datagram, encoded as JSON value. This is really [`IbcSpec::Datagram`],
    /// and will be interpreted based on the implementation defined by [`Self::ibc_spec_id`].
    pub datagram: Value,
}

impl IbcDatagram {
    pub fn decode_datagram<V: IbcSpec>(&self) -> Option<Result<V::Datagram, serde_json::Error>> {
        if self.ibc_spec_id == V::ID {
            Some(serde_json::from_value(self.datagram.clone()))
        } else {
            None
        }
    }

    // TODO: Make this accept Into<V::Datagram>?
    pub fn new<V: IbcSpec>(datagram: V::Datagram) -> Self {
        Self {
            ibc_spec_id: V::ID,
            datagram: into_value(datagram),
        }
    }
}

#[model]
pub struct UnfinalizedTrustedClientState {
    pub height: Height,
    pub client_state: ClientStateMeta,
}

#[model]
pub struct DecodedHeaderMeta {
    /// The new trusted height that the header provides a consensus update to.
    pub height: Height,
}

// client update plugins produce this data which is then used when constructing the OrderedClientUpdates
#[model]
pub struct OrderedHeaders {
    pub headers: Vec<(DecodedHeaderMeta, Value)>,
}

#[model]
pub struct OrderedClientUpdates {
    pub updates: Vec<(DecodedHeaderMeta, ClientUpdate)>,
}

#[model]
pub struct ClientUpdate {
    pub client_id: RawClientId,
    pub ibc_spec_id: IbcSpecId,
    pub client_message: Bytes,
}
