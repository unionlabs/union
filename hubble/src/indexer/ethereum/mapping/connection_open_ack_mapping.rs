use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, mapping::decoder::Decoder},
    event::{connection_open_ack_event::ConnectionOpenAckEvent, supported::SupportedBlockEvent},
};

impl EthFetcherClient {
    pub fn to_connection_open_ack(
        &self,
        decoder: &Decoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_connection_open_ack - {decoder}");

        Ok(vec![SupportedBlockEvent::ConnectionOpenAck {
            inner: ConnectionOpenAckEvent {
                header: decoder.header()?,
                connection_id: decoder.event.connection_id()?,
                client_id: decoder.event.client_id()?,
                counterparty_client_id: decoder.event.counterparty_client_id()?,
                counterparty_connection_id: decoder.event.counterparty_connection_id()?,
            },
        }])
    }
}
