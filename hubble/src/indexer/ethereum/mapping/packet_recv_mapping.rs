use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    ethereum::{fetcher_client::EthFetcherClient, mapping::decoder::Decoder},
    event::{packet_recv_event::PacketRecvEvent, supported::SupportedBlockEvent},
};

impl EthFetcherClient {
    pub fn to_packet_recv(
        &self,
        decoder: &Decoder,
    ) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_packet_recv - {decoder}");

        Ok(vec![SupportedBlockEvent::PacketRecv {
            inner: PacketRecvEvent {
                header: decoder.header()?,
                channel_id: decoder.event.channel_id()?,
                packet_hash: decoder.event.packet_hash()?,
                maker: decoder.event.maker()?,
                maker_msg: decoder.event.maker_msg()?,
            },
        }])
    }
}
