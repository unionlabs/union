use serde::{Deserialize, Serialize};

use crate::indexer::{
    event::{
        header::Header,
        types::{ChannelId, Denom, Path},
    },
    handler::types::{CreateWrappedTokenKind, Metadata},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CreateWrappedTokenEvent {
    #[serde(flatten)]
    pub header: Header,
    pub path: Path,
    pub channel_id: ChannelId,
    pub base_token: Denom,
    pub quote_token: Denom,
    pub metadata: Metadata,
    pub kind: CreateWrappedTokenKind,
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::*;
    use crate::indexer::event::test_utils::test_helpers::{
        create_test_header, test_json_format, test_roundtrip_serialization,
    };

    /// Creates a test CreateWrappedTokenEvent with predictable values
    fn create_test_event(suffix: u32) -> CreateWrappedTokenEvent {
        let header = create_test_header(suffix);

        CreateWrappedTokenEvent {
            header,
            channel_id: ChannelId(42),
            path: Path(42_u128.try_into().unwrap()),
            base_token: Denom(Bytes::from("base-token")),
            quote_token: Denom(Bytes::from("quote-token")),
            metadata: Metadata(Bytes::from("metadata")),
            kind: CreateWrappedTokenKind::Protocol,
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
  "base_token": "0x626173652d746f6b656e",
  "block_hash": "0x424c4f434b5f484153485f3432",
  "channel_id": 42,
  "event_index": "42",
  "height": "10042",
  "message_index": "542",
  "path": "0x2a",
  "quote_token": "0x71756f74652d746f6b656e",
  "timestamp": "2020-09-13T12:27:22Z",
  "transaction_event_index": "242",
  "transaction_hash": "0x54585f484153485f3432",
  "transaction_index": "142",
  "universal_chain_id": "test-chain-42",
  "metadata": "0x6d65746164617461",
  "kind": 0
}"#;

        test_json_format(&event, expected_json);
    }
}
