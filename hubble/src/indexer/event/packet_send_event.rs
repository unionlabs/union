use serde::{Deserialize, Serialize};

use crate::indexer::event::{
    header::Header,
    types::{BlockHeight, ChannelId, PacketData, PacketHash, TimeoutTimestamp},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PacketSendEvent {
    #[serde(flatten)]
    pub header: Header,
    pub channel_id: ChannelId,
    pub packet_hash: PacketHash,
    pub source_channel_id: ChannelId,
    pub destination_channel_id: ChannelId,
    pub timeout_height: BlockHeight,
    pub timeout_timestamp: TimeoutTimestamp,
    pub data: PacketData,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::event::test_utils::test_helpers::{
        create_packet_send_test_values, create_test_header, test_json_format,
        test_roundtrip_serialization,
    };

    /// Creates a test PacketSendEvent with predictable values
    fn create_test_event(suffix: u32) -> PacketSendEvent {
        let header = create_test_header(suffix);
        let (
            channel_id,
            packet_hash,
            source_channel_id,
            destination_channel_id,
            timeout_height,
            timeout_timestamp,
            data,
        ) = create_packet_send_test_values(suffix);

        PacketSendEvent {
            header,
            channel_id,
            packet_hash,
            source_channel_id,
            destination_channel_id,
            timeout_height,
            timeout_timestamp,
            data,
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
  "block_hash": "0x424c4f434b5f484153485f3432",
  "channel_id": 1042,
  "data": "0x7061636b65742d646174612d3432",
  "destination_channel_id": 3042,
  "event_index": "42",
  "height": "10042",
  "message_index": "542",
  "packet_hash": "0x7061636b65742d686173682d3432",
  "source_channel_id": 2042,
  "timeout_height": "10042",
  "timeout_timestamp": "5000042",
  "timestamp": "2020-09-13T12:27:22Z",
  "transaction_event_index": "242",
  "transaction_hash": "0x54585f484153485f3432",
  "transaction_index": "142",
  "universal_chain_id": "test-chain-42"
}"#;

        test_json_format(&event, expected_json);
    }
}
