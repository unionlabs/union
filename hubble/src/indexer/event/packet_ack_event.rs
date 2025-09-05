use serde::{Deserialize, Serialize};

use crate::indexer::event::{
    header::Header,
    types::{Acknowledgement, ChannelId, Maker, PacketHash},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PacketAckEvent {
    #[serde(flatten)]
    pub header: Header,
    pub channel_id: ChannelId,
    pub packet_hash: PacketHash,
    pub acknowledgement: Acknowledgement,
    pub maker: Maker,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::event::test_utils::test_helpers::{
        create_packet_test_values, create_test_header, test_json_format,
        test_roundtrip_serialization,
    };

    /// Creates a test PacketAckEvent with predictable values
    fn create_test_event(suffix: u32) -> PacketAckEvent {
        let header = create_test_header(suffix);
        let (channel_id, packet_hash, maker, _maker_msg, acknowledgement) =
            create_packet_test_values(suffix);

        PacketAckEvent {
            header,
            channel_id,
            packet_hash,
            acknowledgement,
            maker,
        }
    }

    #[test]
    fn test_json_serialization() {
        let event = create_test_event(42);
        test_roundtrip_serialization(&event);
    }

    #[test]
    fn test_json_format_stability() {
        let event = create_test_event(42);

        let expected_json = r#"{
  "acknowledgement": "0x61636b2d3432",
  "block_hash": "0x424c4f434b5f484153485f3432",
  "channel_id": 1042,
  "event_index": "42",
  "height": "10042",
  "message_index": "542",
  "maker": "0x6d616b65722d3432",
  "packet_hash": "0x7061636b65742d686173682d3432",
  "timestamp": "2020-09-13T12:27:22Z",
  "transaction_event_index": "242",
  "transaction_hash": "0x54585f484153485f3432",
  "transaction_index": "142",
  "universal_chain_id": "test-chain-42"
}"#;

        test_json_format(&event, expected_json);
    }
}
