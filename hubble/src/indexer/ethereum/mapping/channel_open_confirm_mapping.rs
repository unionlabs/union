use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, log_decoder::LogDecoder},
    event::{channel_open_confirm_event::ChannelOpenConfirmEvent, supported::SupportedBlockEvent},
};

impl EthFetcherClient {
    pub fn to_channel_open_confirm(
        &self,
        log: &LogDecoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_channel_open_confirm - {log}");

        Ok(vec![SupportedBlockEvent::ChannelOpenConfirm {
            inner: ChannelOpenConfirmEvent {
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
