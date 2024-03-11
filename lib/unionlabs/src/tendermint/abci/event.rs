use macros::proto;

use crate::tendermint::abci::event_attribute::EventAttribute;

#[derive(Debug, Clone, PartialEq)]
#[proto(raw = protos::tendermint::abci::Event, into, from)]
pub struct Event {
    pub ty: String,
    pub attributes: Vec<EventAttribute>,
}

impl From<protos::tendermint::abci::Event> for Event {
    fn from(value: protos::tendermint::abci::Event) -> Self {
        Self {
            ty: value.r#type,
            attributes: value.attributes.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Event> for protos::tendermint::abci::Event {
    fn from(value: Event) -> Self {
        Self {
            r#type: value.ty,
            attributes: value.attributes.into_iter().map(Into::into).collect(),
        }
    }
}
