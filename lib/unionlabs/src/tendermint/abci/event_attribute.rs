use crate::{Proto, TypeUrl};

#[derive(Debug, Clone, PartialEq)]
pub struct EventAttribute {
    pub key: String,
    pub value: String,
    /// nondeterministic
    pub index: bool,
}

impl Proto for EventAttribute {
    type Proto = protos::tendermint::abci::EventAttribute;
}

impl TypeUrl for protos::tendermint::abci::EventAttribute {
    const TYPE_URL: &'static str = "/tendermint.abci.EventAttribute";
}

impl From<protos::tendermint::abci::EventAttribute> for EventAttribute {
    fn from(value: protos::tendermint::abci::EventAttribute) -> Self {
        Self {
            key: value.key,
            value: value.value,
            index: value.index,
        }
    }
}

impl From<EventAttribute> for protos::tendermint::abci::EventAttribute {
    fn from(value: EventAttribute) -> Self {
        Self {
            key: value.key,
            value: value.value,
            index: value.index,
        }
    }
}
