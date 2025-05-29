use enumorph::Enumorph;
use macros::model;
use serde::de::DeserializeOwned;
use serde_json::Value;
use subset_of::SubsetOf;
use unionlabs::{ibc::core::client::height::Height, primitives::H256, traits::Member};
use voyager_primitives::{ChainId, ClientInfo, IbcSpec, IbcSpecId};

use crate::PluginMessage;

#[model]
#[derive(Enumorph, SubsetOf)]
#[allow(clippy::large_enum_variant)]
pub enum Data {
    IbcEvent(ChainEvent),
    IbcDatagram(IbcDatagram),

    OrderedHeaders(OrderedHeaders),
    // OrderedClientUpdates(OrderedClientUpdates),
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

    pub fn into_ibc_event(self) -> Option<ChainEvent> {
        if let Self::IbcEvent(v) = self {
            Some(v)
        } else {
            None
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
    pub provable_height: EventProvableHeight,

    pub ibc_spec_id: IbcSpecId,
    /// The full IBC event, encoded as JSON value. This is really [`IbcSpec::Event`],
    /// and will be interpreted based on the implementation defined by [`Self::ibc_spec_id`].
    pub event: Value,
}

#[model]
#[derive(Copy)]
pub enum EventProvableHeight {
    /// This event is provable at any height >= this height.
    ///
    /// This is the standard behaviour for chains with fully merkleized state.
    Min(Height),
    /// This event is provable at *exactly* this height.
    Exactly(Height),
}

impl EventProvableHeight {
    pub fn height(&self) -> &Height {
        match self {
            EventProvableHeight::Min(height) => height,
            EventProvableHeight::Exactly(height) => height,
        }
    }
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
    pub fn new<V: IbcSpec>(datagram: impl Into<V::Datagram>) -> Self {
        Self {
            ibc_spec_id: V::ID,
            datagram: serde_json::to_value(datagram.into()).unwrap(),
        }
    }
}

#[model]
pub struct DecodedHeaderMeta {
    /// The new trusted height that the header provides a consensus update to.
    pub height: Height,
}

// client update plugins produce this data which is then used when constructing the
// OrderedClientUpdates
#[model]
pub struct OrderedHeaders {
    pub headers: Vec<(DecodedHeaderMeta, Value)>,
}

// #[model]
// pub struct OrderedClientUpdates {
//     pub updates: Vec<(DecodedHeaderMeta, ClientUpdate)>,
// }

// #[model]
// pub struct ClientUpdate {
//     pub client_id: RawClientId,
//     pub ibc_spec_id: IbcSpecId,
//     pub client_message: Bytes,
// }
