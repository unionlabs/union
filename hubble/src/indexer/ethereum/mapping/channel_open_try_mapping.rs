use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, log_decoder::LogDecoder},
    event::{channel_open_try_event::ChannelOpenTryEvent, supported::SupportedBlockEvent},
};

impl EthFetcherClient {
    pub fn to_channel_open_try(
        &self,
        log: &LogDecoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_channel_open_try - {log}");

        Ok(vec![SupportedBlockEvent::ChannelOpenTry {
            inner: ChannelOpenTryEvent {
                header: log.header()?,
                connection_id: log.connection_id()?,
                channel_id: log.channel_id()?,
                port_id: log.port_id()?,
                counterparty_port_id: log.counterparty_port_id()?,
                counterparty_channel_id: log.counterparty_channel_id()?,
                counterparty_version: log.counterparty_version()?,
            },
        }])
    }
}
