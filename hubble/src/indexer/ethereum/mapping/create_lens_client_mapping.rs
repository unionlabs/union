use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, mapping::decoder::Decoder},
    event::{create_lens_client_event::CreateLensClientEvent, supported::SupportedBlockEvent},
};

impl EthFetcherClient {
    pub fn to_create_lens_client(
        &self,
        decoder: &Decoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_create_lens_client - {decoder}");

        Ok(vec![SupportedBlockEvent::CreateLensClient {
            inner: CreateLensClientEvent {
                header: decoder.header()?,
                client_id: decoder.event.client_id()?,
                l1_client_id: decoder.event.l1_client_id()?,
                l2_client_id: decoder.event.l2_client_id()?,
                l2_chain_id: decoder.event.l2_chain_id()?,
            },
        }])
    }
}
