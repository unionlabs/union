use serde::{Deserialize, Serialize};

use crate::indexer::event::{
    header::Header,
    types::{Capacity, Denom, RefillRate},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TokenBucketUpdateEvent {
    #[serde(flatten)]
    pub header: Header,
    pub denom: Denom,
    pub capacity: Capacity,
    pub refill_rate: RefillRate,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::event::test_utils::test_helpers::{
        create_test_header, create_token_bucket_test_values, test_json_format,
        test_roundtrip_serialization,
    };

    /// Creates a test event with unique deterministic values
    fn create_test_event(suffix: u32) -> TokenBucketUpdateEvent {
        let header = create_test_header(suffix);
        let (denom, capacity, refill_rate) = create_token_bucket_test_values(suffix);

        TokenBucketUpdateEvent {
            header,
            denom,
            capacity,
            refill_rate,
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
  "capacity": "0xf426a",
  "denom": "0x64656e6f6d2d3432",
  "event_index": "42",
  "height": "10042",
  "message_index": "542",
  "refill_rate": "0x8e",
  "timestamp": "2020-09-13T12:27:22Z",
  "transaction_event_index": "242",
  "transaction_hash": "0x54585f484153485f3432",
  "transaction_index": "142",
  "universal_chain_id": "test-chain-42"
}"#;
        test_json_format(&event, expected_json);
    }
}
