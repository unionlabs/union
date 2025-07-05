use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, mapping::decoder::Decoder},
    event::{connection_open_init_event::ConnectionOpenInitEvent, supported::SupportedBlockEvent},
};

impl EthFetcherClient {
    pub fn to_connection_open_init(
        &self,
        decoder: &Decoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_connection_open_init - {decoder}");

        Ok(vec![SupportedBlockEvent::ConnectionOpenInit {
            inner: ConnectionOpenInitEvent {
                header: decoder.header()?,
                connection_id: decoder.event.connection_id()?,
                client_id: decoder.event.client_id()?,
                counterparty_client_id: decoder.event.counterparty_client_id()?,
            },
        }])
    }
}
