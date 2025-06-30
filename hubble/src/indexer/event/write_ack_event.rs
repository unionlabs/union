use serde::{Deserialize, Serialize};

use crate::indexer::event::{
    header::Header,
    types::{Acknowledgement, ChannelId, PacketHash},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct WriteAckEvent {
    #[serde(flatten)]
    pub header: Header,
    pub channel_id: ChannelId,
    pub packet_hash: PacketHash,
    pub acknowledgement: Acknowledgement,
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::*;
    use crate::indexer::event::test_utils::test_helpers::*;

    /// Creates a test WriteAckEvent with unique deterministic values
    fn create_test_event(suffix: u32) -> WriteAckEvent {
        WriteAckEvent {
            header: create_test_header(suffix),
            channel_id: ChannelId(suffix + 1000),
            packet_hash: PacketHash(Bytes::from(format!("packet-hash-{}", suffix))),
            acknowledgement: Acknowledgement(Bytes::from(format!("ack-{}", suffix))),
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

        // Test that the JSON format matches expectations

        let expected_json = r#"{
  "acknowledgement": "0x61636b2d3432",
  "block_hash": "0x424c4f434b5f484153485f3432",
  "channel_id": 1042,
  "event_index": "42",
  "height": "10042",
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
