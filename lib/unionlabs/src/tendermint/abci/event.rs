use crate::{tendermint::abci::event_attribute::EventAttribute, Proto, TypeUrl};

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub ty: String,
    pub attributes: Vec<EventAttribute>,
}

impl Proto for Event {
    type Proto = protos::tendermint::abci::Event;
}

impl TypeUrl for protos::tendermint::abci::Event {
    const TYPE_URL: &'static str = "/tendermint.abci.Event";
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
