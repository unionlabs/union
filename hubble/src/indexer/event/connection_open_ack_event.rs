use serde::{Deserialize, Serialize};

use crate::indexer::event::{
    header::Header,
    types::{ClientId, ConnectionId},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ConnectionOpenAckEvent {
    #[serde(flatten)]
    pub header: Header,
    pub connection_id: ConnectionId,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::event::test_utils::test_helpers::{
        create_connection_test_values, create_test_header, test_json_format,
        test_roundtrip_serialization,
    };

    /// Creates a test ConnectionOpenAckEvent with unique deterministic values
    fn create_test_event(suffix: u32) -> ConnectionOpenAckEvent {
        let header = create_test_header(suffix);
        let (connection_id, client_id, counterparty_client_id, counterparty_connection_id) =
            create_connection_test_values(suffix);

        ConnectionOpenAckEvent {
            header,
            connection_id,
            client_id,
            counterparty_client_id,
            counterparty_connection_id,
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
  "universal_chain_id": "test-chain-42",
  "block_hash": "0x424c4f434b5f484153485f3432",
  "height": "10042",
  "event_index": "42",
  "timestamp": "2020-09-13T12:27:22Z",
  "transaction_hash": "0x54585f484153485f3432",
  "transaction_index": "142",
  "transaction_event_index": "242",
  "connection_id": 1042,
  "client_id": 2042,
  "counterparty_client_id": 3042,
  "counterparty_connection_id": 4042
}"#;
        test_json_format(&event, expected_json);
    }
}
