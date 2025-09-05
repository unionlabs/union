use serde::{Deserialize, Serialize};

use crate::indexer::event::{
    header::Header,
    types::{ChannelId, ChannelVersion, ConnectionId, PortId},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ChannelOpenInitEvent {
    #[serde(flatten)]
    pub header: Header,
    pub connection_id: ConnectionId,
    pub channel_id: ChannelId,
    pub port_id: PortId,
    pub counterparty_port_id: PortId,
    pub version: ChannelVersion,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::event::{
        test_utils::test_helpers::{
            create_channel_test_values, create_test_header, test_json_format,
            test_roundtrip_serialization,
        },
        types::ChannelVersion,
    };

    /// Creates a test ChannelOpenInitEvent with unique deterministic values
    fn create_test_event(suffix: u32) -> ChannelOpenInitEvent {
        let header = create_test_header(suffix);
        let (connection_id, channel_id, port_id, _counterparty_channel_id, counterparty_port_id) =
            create_channel_test_values(suffix);

        ChannelOpenInitEvent {
            header,
            connection_id,
            channel_id,
            port_id,
            counterparty_port_id,
            version: ChannelVersion(format!("version-{}", suffix)),
        }
    }

    #[test]
    fn test_json_serialization() {
        let event = create_test_event(1);
        test_roundtrip_serialization(&event);
    }

    #[test]
    fn test_json_format_stability() {
        let event = create_test_event(42);
        let expected_json = r#"{
  "block_hash": "0x424c4f434b5f484153485f3432",
  "channel_id": 2042,
  "connection_id": 1042,
  "counterparty_port_id": "0x636f756e746572706172742d3432",
  "event_index": "42",
  "height": "10042",
  "message_index": "542",
  "port_id": "0x706f72742d3432",
  "timestamp": "2020-09-13T12:27:22Z",
  "transaction_event_index": "242",
  "transaction_hash": "0x54585f484153485f3432",
  "transaction_index": "142",
  "universal_chain_id": "test-chain-42",
  "version": "version-42"
}"#;
        test_json_format(&event, expected_json);
    }
}
