use serde::{Deserialize, Serialize};

use crate::indexer::event::{
    header::Header,
    types::{CanonicalChainId, ClientId},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CreateLensClientEvent {
    #[serde(flatten)]
    pub header: Header,
    pub client_id: ClientId,
    pub l1_client_id: ClientId,
    pub l2_client_id: ClientId,
    pub l2_chain_id: CanonicalChainId,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::event::{
        test_utils::test_helpers::{
            create_test_header, test_json_format, test_roundtrip_serialization,
        },
        types::{CanonicalChainId, ClientId},
    };

    /// Creates a test CreateLensClientEvent with unique deterministic values
    fn create_test_event(suffix: u32) -> CreateLensClientEvent {
        let header = create_test_header(suffix);

        CreateLensClientEvent {
            header,
            client_id: ClientId(suffix + 1000),
            l1_client_id: ClientId(suffix + 2000),
            l2_client_id: ClientId(suffix + 3000),
            l2_chain_id: CanonicalChainId(format!("l2-chain-{}", suffix)),
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
  "message_index": "542",
  "event_index": "42",
  "timestamp": "2020-09-13T12:27:22Z",
  "transaction_hash": "0x54585f484153485f3432",
  "transaction_index": "142",
  "transaction_event_index": "242",
  "client_id": 1042,
  "l1_client_id": 2042,
  "l2_client_id": 3042,
  "l2_chain_id": "l2-chain-42"
}"#;
        test_json_format(&event, expected_json);
    }
}
