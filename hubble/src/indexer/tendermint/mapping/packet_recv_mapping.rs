use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{packet_recv_event::PacketRecvEvent, supported::SupportedBlockEvent},
    tendermint::{fetcher_client::TmFetcherClient, mapping::decoder::Decoder},
};

impl TmFetcherClient {
    pub fn to_packet_recv(&self, log: &Decoder) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
        trace!("to_packet_recv - {log}");

        Ok(vec![SupportedBlockEvent::PacketRecv {
            inner: PacketRecvEvent {
                header: log.header()?,
                channel_id: log.event.channel_id()?,
                packet_hash: log.event.packet_hash()?,
                maker: log.event.maker()?,
                maker_msg: log.event.maker_msg()?,
            },
        }])
    }
}
