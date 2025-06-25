use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, log_decoder::LogDecoder},
    event::{channel_open_init_event::ChannelOpenInitEvent, supported::SupportedBlockEvent},
};

impl EthFetcherClient {
    pub fn to_channel_open_init(
        &self,
        log: &LogDecoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_channel_open_init - {log}");

        Ok(vec![SupportedBlockEvent::ChannelOpenInit {
            inner: ChannelOpenInitEvent {
                header: log.header()?,
                connection_id: log.connection_id()?,
                channel_id: log.channel_id()?,
                port_id: log.port_id()?,
                counterparty_port_id: log.counterparty_port_id()?,
                version: log.version()?,
            },
        }])
    }
}
