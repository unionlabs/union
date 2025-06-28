use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, mapping::decoder::Decoder},
    event::{supported::SupportedBlockEvent, write_ack_event::WriteAckEvent},
};

impl EthFetcherClient {
    pub fn to_write_ack(
        &self,
        decoder: &Decoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_write_ack - {decoder}");

        Ok(vec![SupportedBlockEvent::WriteAck {
            inner: WriteAckEvent {
                header: decoder.header()?,
                channel_id: decoder.event.channel_id()?,
                packet_hash: decoder.event.packet_hash()?,
                acknowledgement: decoder.event.acknowledgement()?,
            },
        }])
    }
}
