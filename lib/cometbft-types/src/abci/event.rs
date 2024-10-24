use serde::{Deserialize, Serialize};

use crate::abci::event_attribute::EventAttribute;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub ty: String,
    pub attributes: Vec<EventAttribute>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::abci::event::Event;

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
}
