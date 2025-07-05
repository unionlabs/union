use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, mapping::decoder::Decoder},
    event::{packet_timeout_event::PacketTimeoutEvent, supported::SupportedBlockEvent},
};

impl EthFetcherClient {
    pub fn to_packet_timeout(
        &self,
        decoder: &Decoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_packet_timeout - {decoder}");

        Ok(vec![SupportedBlockEvent::PacketTimeout {
            inner: PacketTimeoutEvent {
                header: decoder.header()?,
                channel_id: decoder.event.channel_id()?,
                packet_hash: decoder.event.packet_hash()?,
                maker: decoder.event.maker()?,
            },
        }])
    }
}
