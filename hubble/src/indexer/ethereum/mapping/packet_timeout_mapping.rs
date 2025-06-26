use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, log_decoder::LogDecoder},
    event::{packet_timeout_event::PacketTimeoutEvent, supported::SupportedBlockEvent},
};

impl EthFetcherClient {
    pub fn to_packet_timeout(
        &self,
        log: &LogDecoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_packet_timeout - {log}");

        Ok(vec![SupportedBlockEvent::PacketTimeout {
            inner: PacketTimeoutEvent {
                header: log.header()?,
                channel_id: log.event.channel_id()?,
                packet_hash: log.event.packet_hash()?,
                maker: log.event.maker()?,
            },
        }])
    }
}
