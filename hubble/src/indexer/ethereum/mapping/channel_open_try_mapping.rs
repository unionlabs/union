use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, mapping::decoder::Decoder},
    event::{channel_open_try_event::ChannelOpenTryEvent, supported::SupportedBlockEvent},
};

impl EthFetcherClient {
    pub fn to_channel_open_try(
        &self,
        decoder: &Decoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_channel_open_try - {decoder}");

        Ok(vec![SupportedBlockEvent::ChannelOpenTry {
            inner: ChannelOpenTryEvent {
                header: decoder.header()?,
                connection_id: decoder.event.connection_id()?,
                channel_id: decoder.event.channel_id()?,
                port_id: decoder.event.port_id()?,
                counterparty_port_id: decoder.event.counterparty_port_id()?,
                counterparty_channel_id: decoder.event.counterparty_channel_id()?,
                counterparty_version: decoder.event.counterparty_version()?,
            },
        }])
    }
}
