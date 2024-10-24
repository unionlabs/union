use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventAttribute {
    pub key: String,
    pub value: String,
    /// nondeterministic
    pub index: bool,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::abci::event_attribute::EventAttribute;

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
}
