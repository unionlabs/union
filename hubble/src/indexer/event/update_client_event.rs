use serde::{Deserialize, Serialize};

use crate::indexer::event::{
    header::Header,
    types::{BlockHeight, ClientId},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct UpdateClientEvent {
    #[serde(flatten)]
    pub header: Header,
    pub client_id: ClientId,
    pub counterparty_height: BlockHeight,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::event::{
        test_utils::test_helpers::{
            create_test_header, test_json_format, test_roundtrip_serialization,
        },
        types::{BlockHeight, ClientId},
    };

    /// Creates a test UpdateClientEvent with unique deterministic values
    fn create_test_event(suffix: u32) -> UpdateClientEvent {
        let header = create_test_header(suffix);

        UpdateClientEvent {
            header,
            client_id: ClientId(suffix + 1000),
            counterparty_height: BlockHeight(suffix as u64 + 20000),
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
  "client_id": 1042,
  "counterparty_height": "20042"
}"#;
        test_json_format(&event, expected_json);
    }
}
