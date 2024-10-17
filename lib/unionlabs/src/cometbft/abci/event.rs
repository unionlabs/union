use macros::model;

use crate::cometbft::abci::event_attribute::EventAttribute;

#[model(proto(raw(protos::cometbft::abci::v1::Event), into, from))]
pub struct Event {
    pub ty: String,
    pub attributes: Vec<EventAttribute>,
}

impl From<protos::cometbft::abci::v1::Event> for Event {
    fn from(value: protos::cometbft::abci::v1::Event) -> Self {
        Self {
            ty: value.r#type,
            attributes: value.attributes.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Event> for protos::cometbft::abci::v1::Event {
    fn from(value: Event) -> Self {
        Self {
            r#type: value.ty,
            attributes: value.attributes.into_iter().map(Into::into).collect(),
        }
    }
}
