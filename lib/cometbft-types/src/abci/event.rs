use serde::{Deserialize, Serialize};

use crate::abci::event_attribute::EventAttribute;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    #[serde(rename = "type")]
    pub ty: String,
    pub attributes: Vec<EventAttribute>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::abci::event::Event;

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
}
