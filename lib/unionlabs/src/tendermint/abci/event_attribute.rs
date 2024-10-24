use macros::model;

#[model(proto(raw(protos::tendermint::abci::EventAttribute), into, from))]
pub struct EventAttribute {
    pub key: String,
    pub value: String,
    /// nondeterministic
    pub index: bool,
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
