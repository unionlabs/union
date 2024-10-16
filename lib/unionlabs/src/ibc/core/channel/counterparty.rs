use macros::model;

use crate::id::{ChannelId, PortId};

#[model(proto(raw(protos::ibc::core::channel::v1::Counterparty), into, from))]
#[cfg_attr(feature = "valuable", derive(valuable::Valuable))]
pub struct Counterparty {
    pub port_id: PortId,
    pub channel_id: Option<ChannelId>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        ibc::core::channel::counterparty::Counterparty,
        id::{ChannelId, Ics24IdParseError, ParsePrefixedIdError},
    };

    impl From<Counterparty> for protos::ibc::core::channel::v1::Counterparty {
        fn from(value: Counterparty) -> Self {
            Self {
                port_id: value.port_id.to_string(),
                channel_id: value
                    .channel_id
                    .map_or_else(String::new, |channel_id| channel_id.to_string_prefixed()),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum TryFromChannelCounterpartyError {
        #[error("invalid port_id")]
        PortId(#[source] Ics24IdParseError),
        #[error("invalid channel_id")]
        ChannelId(#[source] ParsePrefixedIdError),
    }

    impl TryFrom<protos::ibc::core::channel::v1::Counterparty> for Counterparty {
        type Error = TryFromChannelCounterpartyError;

        fn try_from(
            proto: protos::ibc::core::channel::v1::Counterparty,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                port_id: proto
                    .port_id
                    .parse()
                    .map_err(TryFromChannelCounterpartyError::PortId)?,
                channel_id: if proto.channel_id.is_empty() {
                    None
                } else {
                    ChannelId::parse_prefixed(&proto.channel_id)
                        .map(Some)
                        .map_err(TryFromChannelCounterpartyError::ChannelId)?
                },
            })
        }
    }
}
