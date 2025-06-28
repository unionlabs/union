use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{packet_timeout_event::PacketTimeoutEvent, supported::SupportedBlockEvent},
    tendermint::{fetcher_client::TmFetcherClient, mapping::decoder::Decoder},
};

impl TmFetcherClient {
    pub fn to_packet_timeout(
        &self,
        log: &Decoder,
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
