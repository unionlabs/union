use macros::model;

#[model(proto(raw(protos::cometbft::abci::v1::EventAttribute), into, from))]
pub struct EventAttribute {
    pub key: String,
    pub value: String,
    /// nondeterministic
    pub index: bool,
}

impl From<protos::cometbft::abci::v1::EventAttribute> for EventAttribute {
    fn from(value: protos::cometbft::abci::v1::EventAttribute) -> Self {
        Self {
            key: value.key,
            value: value.value,
            index: value.index,
        }
    }
}

impl From<EventAttribute> for protos::cometbft::abci::v1::EventAttribute {
    fn from(value: EventAttribute) -> Self {
        Self {
            key: value.key,
            value: value.value,
            index: value.index,
        }
    }
}
