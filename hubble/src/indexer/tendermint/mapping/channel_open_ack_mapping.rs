use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{channel_open_ack_event::ChannelOpenAckEvent, supported::SupportedBlockEvent},
    tendermint::{fetcher_client::TmFetcherClient, mapping::decoder::Decoder},
};

impl TmFetcherClient {
    pub fn to_channel_open_ack(
        &self,
        event: &Decoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_channel_open_ack - {event}");

        Ok(vec![SupportedBlockEvent::ChannelOpenAck {
            inner: ChannelOpenAckEvent {
                header: event.header()?,
                connection_id: event.event.connection_id()?,
                channel_id: event.event.channel_id()?,
                port_id: event.event.port_id()?,
                counterparty_port_id: event.event.counterparty_port_id()?,
                counterparty_channel_id: event.event.counterparty_channel_id()?,
            },
        }])
    }
}
