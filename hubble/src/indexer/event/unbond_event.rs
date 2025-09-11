use serde::{Deserialize, Serialize};

use crate::indexer::event::{
    header::Header,
    types::{Batch, UnbondAmount, UnbondIsNewRequest, UnbondStakerAddress},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct UnbondEvent {
    #[serde(flatten)]
    pub header: Header,
    pub amount: UnbondAmount,
    pub batch: Batch,
    pub is_new_request: UnbondIsNewRequest,
    pub staker: UnbondStakerAddress,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::event::test_utils::test_helpers::{
        create_test_header, create_unbond_test_values, test_json_format,
        test_roundtrip_serialization,
    };

    /// Creates a test UnbondEvent with predictable values
    fn create_test_event(suffix: u32) -> UnbondEvent {
        let header = create_test_header(suffix);
        let (amount, batch, is_new_request, staker) = create_unbond_test_values(42);

        UnbondEvent {
            header,
            amount,
            batch,
            is_new_request,
            staker,
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
            "amount": "542",
            "batch": "1042",
            "block_hash": "0x424c4f434b5f484153485f3432",
            "event_index": "42",
            "height": "10042",
            "is_new_request": true,
            "message_index": "542",
            "staker": "0x756e626f6e642d7374616b65722d3432",
            "timestamp": "2020-09-13T12:27:22Z",
            "transaction_event_index": "242",
            "transaction_hash": "0x54585f484153485f3432",
            "transaction_index": "142",
            "universal_chain_id": "test-chain-42"
        }"#;

        test_json_format(&event, expected_json);
    }
}
