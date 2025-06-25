use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, log_decoder::LogDecoder},
    event::{connection_open_try_event::ConnectionOpenTryEvent, supported::SupportedBlockEvent},
};

impl EthFetcherClient {
    pub fn to_connection_open_try(
        &self,
        log: &LogDecoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_connection_open_try - {log}");

        Ok(vec![SupportedBlockEvent::ConnectionOpenTry {
            inner: ConnectionOpenTryEvent {
                header: log.header()?,
                connection_id: log.connection_id()?,
                client_id: log.client_id()?,
                counterparty_client_id: log.counterparty_client_id()?,
                counterparty_connection_id: log.counterparty_connection_id()?,
            },
        }])
    }
}
