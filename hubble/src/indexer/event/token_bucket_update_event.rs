use serde::{Deserialize, Serialize};

use crate::indexer::event::{
    header::Header,
    types::{Capacity, Denom, RefillRate},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TokenBucketUpdateEvent {
    #[serde(flatten)]
    pub header: Header,
    pub denom: Denom,
    pub capacity: Capacity,
    pub refill_rate: RefillRate,
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use time::OffsetDateTime;

    use super::*;
    use crate::indexer::event::{
        header::Header,
        types::{
            BlockHash, BlockHeight, BlockTimestamp, Capacity, Denom, EventIndex, RefillRate,
            TransactionEventIndex, TransactionHash, TransactionIndex, UniversalChainId,
        },
    };

    #[test]
    fn test_token_bucket_update_event_json_serialization() {
        // Create a sample TokenBucketUpdateEvent
        let event = TokenBucketUpdateEvent {
            header: Header {
                universal_chain_id: UniversalChainId("cosmoshub-4".to_string()),
                block_hash: BlockHash(Bytes::from_static(b"ABC123DEF456")),
                height: BlockHeight(12345),
                event_index: EventIndex(0),
                timestamp: BlockTimestamp(OffsetDateTime::from_unix_timestamp(1640995200).unwrap()),
                transaction_hash: TransactionHash(Bytes::from_static(b"TX123HASH456")),
                transaction_index: TransactionIndex(1),
                transaction_event_index: Some(TransactionEventIndex(2)),
            },
            denom: Denom(Bytes::from_static(b"uatom")),
            capacity: Capacity(1000000.try_into().unwrap()),
            refill_rate: RefillRate(100.try_into().unwrap()),
        };

        // Test serialization
        let json_string = serde_json::to_string(&event)
            .expect("Failed to serialize TokenBucketUpdateEvent to JSON");

        println!("Serialized JSON: {}", json_string);

        // Test deserialization
        let deserialized_event: TokenBucketUpdateEvent = serde_json::from_str(&json_string)
            .expect("Failed to deserialize TokenBucketUpdateEvent from JSON");

        // Verify the deserialized event matches the original
        assert_eq!(event.denom, deserialized_event.denom);
        assert_eq!(event.capacity, deserialized_event.capacity);
        assert_eq!(event.refill_rate, deserialized_event.refill_rate);
        assert_eq!(
            event.header.universal_chain_id,
            deserialized_event.header.universal_chain_id
        );
        assert_eq!(
            event.header.block_hash,
            deserialized_event.header.block_hash
        );
        assert_eq!(event.header.height, deserialized_event.header.height);
        assert_eq!(
            event.header.event_index,
            deserialized_event.header.event_index
        );
        assert_eq!(event.header.timestamp, deserialized_event.header.timestamp);
        assert_eq!(
            event.header.transaction_hash,
            deserialized_event.header.transaction_hash
        );
        assert_eq!(
            event.header.transaction_index,
            deserialized_event.header.transaction_index
        );
        assert_eq!(
            event.header.transaction_event_index,
            deserialized_event.header.transaction_event_index
        );
    }
}
