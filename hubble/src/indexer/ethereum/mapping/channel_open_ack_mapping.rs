use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, mapping::decoder::Decoder},
    event::{channel_open_ack_event::ChannelOpenAckEvent, supported::SupportedBlockEvent},
};

impl EthFetcherClient {
    pub fn to_channel_open_ack(
        &self,
        decoder: &Decoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_channel_open_ack - {decoder}");

        Ok(vec![SupportedBlockEvent::ChannelOpenAck {
            inner: ChannelOpenAckEvent {
                header: decoder.header()?,
                connection_id: decoder.event.connection_id()?,
                channel_id: decoder.event.channel_id()?,
                port_id: decoder.event.port_id()?,
                counterparty_port_id: decoder.event.counterparty_port_id()?,
                counterparty_channel_id: decoder.event.counterparty_channel_id()?,
            },
        }])
    }
}
