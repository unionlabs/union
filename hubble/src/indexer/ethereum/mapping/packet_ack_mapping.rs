use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, mapping::decoder::Decoder},
    event::{packet_ack_event::PacketAckEvent, supported::SupportedBlockEvent},
};

impl EthFetcherClient {
    pub fn to_packet_ack(
        &self,
        decoder: &Decoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_packet_ack - {decoder}");

        Ok(vec![SupportedBlockEvent::PacketAck {
            inner: PacketAckEvent {
                header: decoder.header()?,
                channel_id: decoder.event.channel_id()?,
                packet_hash: decoder.event.packet_hash()?,
                acknowledgement: decoder.event.acknowledgement()?,
                maker: decoder.event.maker()?,
            },
        }])
    }
}
