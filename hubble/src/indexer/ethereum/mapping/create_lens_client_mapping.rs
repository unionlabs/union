use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, log_decoder::LogDecoder},
    event::{create_lens_client_event::CreateLensClientEvent, supported::SupportedBlockEvent},
};

impl EthFetcherClient {
    pub fn to_create_lens_client(
        &self,
        log: &LogDecoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_create_lens_client - {log}");

        Ok(vec![SupportedBlockEvent::CreateLensClient {
            inner: CreateLensClientEvent {
                header: log.header()?,
                client_id: log.client_id()?,
                l1_client_id: log.l1_client_id()?,
                l2_client_id: log.l2_client_id()?,
                l2_chain_id: log.l2_chain_id()?,
            },
        }])
    }
}
