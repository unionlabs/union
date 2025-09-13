use serde::{Deserialize, Serialize};

use crate::indexer::event::{
    header::Header,
    types::{ChannelId, Owner, Path, ProxyAccount},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CreateProxyAccountEvent {
    #[serde(flatten)]
    pub header: Header,
    pub path: Path,
    pub channel_id: ChannelId,
    pub owner: Owner,
    pub proxy_account: ProxyAccount,
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::*;
    use crate::indexer::event::{
        test_utils::test_helpers::{
            create_test_header, test_json_format, test_roundtrip_serialization,
        },
        types::{ChannelId, Owner, Path, ProxyAccount},
    };

    /// Creates a test CreateProxyAccountEvent with predictable values
    fn create_test_event(suffix: u32) -> CreateProxyAccountEvent {
        let header = create_test_header(suffix);

        CreateProxyAccountEvent {
            header,
            channel_id: ChannelId(42),
            path: Path(42_u128.try_into().unwrap()),
            owner: Owner(Bytes::from("owner")),
            proxy_account: ProxyAccount(Bytes::from("proxy_account")),
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
  "channel_id": 42,
  "event_index": "42",
  "height": "10042",
  "message_index": "542",
  "owner": "0x6f776e6572",
  "path": "0x2a",
  "proxy_account": "0x70726f78795f6163636f756e74",
  "timestamp": "2020-09-13T12:27:22Z",
  "transaction_event_index": "242",
  "transaction_hash": "0x54585f484153485f3432",
  "transaction_index": "142",
  "universal_chain_id": "test-chain-42"
}"#;

        test_json_format(&event, expected_json);
    }
}
