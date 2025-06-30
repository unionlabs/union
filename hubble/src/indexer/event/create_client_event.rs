use serde::{Deserialize, Serialize};

use crate::indexer::event::{
    header::Header,
    types::{CanonicalChainId, ClientId, ClientType},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CreateClientEvent {
    #[serde(flatten)]
    pub header: Header,
    pub client_id: ClientId,
    pub client_type: ClientType,
    pub counterparty_chain_id: CanonicalChainId,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::event::test_utils::test_helpers::{
        create_client_test_values, create_test_header, test_json_format,
        test_roundtrip_serialization,
    };

    /// Creates a test CreateClientEvent with unique deterministic values
    fn create_test_event(suffix: u32) -> CreateClientEvent {
        let header = create_test_header(suffix);
        let (client_id, client_type, counterparty_chain_id) = create_client_test_values(suffix);

        CreateClientEvent {
            header,
            client_id,
            client_type,
            counterparty_chain_id,
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
  "client_type": "client-type-42",
  "counterparty_chain_id": "chain-42"
}"#;
        test_json_format(&event, expected_json);
    }
}
