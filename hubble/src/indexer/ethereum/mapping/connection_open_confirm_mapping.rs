use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, log_decoder::LogDecoder},
    event::{
        connection_open_confirm_event::ConnectionOpenConfirmEvent, supported::SupportedBlockEvent,
    },
};

impl EthFetcherClient {
    pub fn to_connection_open_confirm(
        &self,
        log: &LogDecoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_connection_open_confirm - {log}");

        Ok(vec![SupportedBlockEvent::ConnectionOpenConfirm {
            inner: ConnectionOpenConfirmEvent {
                header: log.header()?,
                connection_id: log.event.connection_id()?,
                client_id: log.event.client_id()?,
                counterparty_client_id: log.event.counterparty_client_id()?,
                counterparty_connection_id: log.event.counterparty_connection_id()?,
            },
        }])
    }
}
