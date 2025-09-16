#[cfg(test)]
pub mod test_helpers {
    use bytes::Bytes;
    use serde::{Deserialize, Serialize};
    use time::OffsetDateTime;

    use crate::indexer::event::{
        header::Header,
        types::{
            Acknowledgement, Batch, BlockHash, BlockHeight, BlockTimestamp, BondInAmount,
            BondMintAmount, BondMintToAddress, BondSenderAddress, CanonicalChainId, Capacity,
            ChannelId, ClientId, ClientType, ConnectionId, ContractAddress, Denom, EventIndex,
            Maker, MakerMsg, MessageIndex, MutationAmount, MutationDirection, PacketData,
            PacketHash, PortId, RefillRate, TimeoutTimestamp, TransactionEventIndex,
            TransactionHash, TransactionIndex, UnbondAmount, UnbondIsNewRequest,
            UnbondStakerAddress, UniversalChainId, WalletAddress,
        },
    };

    /// Creates a test Header with unique, deterministic values
    ///
    /// # Arguments
    /// * `suffix` - A unique suffix to differentiate headers in the same test
    pub fn create_test_header(suffix: u32) -> Header {
        Header {
            universal_chain_id: UniversalChainId(format!("test-chain-{}", suffix)),
            block_hash: BlockHash(Bytes::from(format!("BLOCK_HASH_{}", suffix))),
            height: BlockHeight(10000 + suffix as u64),
            event_index: EventIndex(suffix as u64),
            timestamp: BlockTimestamp(
                OffsetDateTime::from_unix_timestamp(1600000000 + suffix as i64)
                    .expect("Valid timestamp"),
            ),
            transaction_hash: TransactionHash(Bytes::from(format!("TX_HASH_{}", suffix))),
            transaction_index: TransactionIndex(100 + suffix as u64),
            transaction_event_index: Some(TransactionEventIndex(200 + suffix as u64)),
            message_index: Some(MessageIndex(500 + suffix as u64)),
        }
    }

    /// Tests JSON serialization/deserialization roundtrip for any type
    ///
    /// # Arguments
    /// * `event` - The event to test
    pub fn test_roundtrip_serialization<T>(event: &T)
    where
        T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
    {
        let json_string = serde_json::to_string(event).expect("Failed to serialize to JSON");
        let deserialized: T =
            serde_json::from_str(&json_string).expect("Failed to deserialize from JSON");
        assert_eq!(*event, deserialized);
    }

    /// Tests JSON format stability by comparing with expected JSON
    ///
    /// # Arguments
    /// * `event` - The event to test
    /// * `expected_json` - The expected JSON string (pretty-printed)
    pub fn test_json_format<T>(event: &T, expected_json: &str)
    where
        T: Serialize,
    {
        let json_string = serde_json::to_string(event).expect("Failed to serialize to JSON");
        let actual_value: serde_json::Value =
            serde_json::from_str(&json_string).expect("Failed to parse actual JSON");
        let expected_value: serde_json::Value =
            serde_json::from_str(expected_json).expect("Failed to parse expected JSON");

        if actual_value != expected_value {
            let actual_pretty = serde_json::to_string_pretty(&actual_value).unwrap();
            let expected_pretty = serde_json::to_string_pretty(&expected_value).unwrap();

            // Find field differences
            let mut diff_details = Vec::new();
            compare_json_values(&actual_value, &expected_value, "", &mut diff_details);

            let diff_summary = if diff_details.is_empty() {
                "No specific field differences found (likely structural difference)".to_string()
            } else {
                format!("Field differences:\n{}", diff_details.join("\n"))
            };

            panic!(
                "JSON format has changed! If this change is intentional, update the expected_json string.\n\n{}\n\nExpected JSON:\n{}\n\nActual JSON:\n{}",
                diff_summary, expected_pretty, actual_pretty
            );
        }
    }

    /// Helper function to recursively compare JSON values and report differences
    fn compare_json_values(
        actual: &serde_json::Value,
        expected: &serde_json::Value,
        path: &str,
        diffs: &mut Vec<String>,
    ) {
        match (actual, expected) {
            (serde_json::Value::Object(actual_obj), serde_json::Value::Object(expected_obj)) => {
                // Check for missing fields in actual
                for (key, expected_val) in expected_obj {
                    let current_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    match actual_obj.get(key) {
                        Some(actual_val) => {
                            compare_json_values(actual_val, expected_val, &current_path, diffs)
                        }
                        None => diffs.push(format!("  Missing field: {}", current_path)),
                    }
                }
                // Check for extra fields in actual
                for key in actual_obj.keys() {
                    if !expected_obj.contains_key(key) {
                        let current_path = if path.is_empty() {
                            key.clone()
                        } else {
                            format!("{}.{}", path, key)
                        };
                        diffs.push(format!("  Extra field: {}", current_path));
                    }
                }
            }
            (actual_val, expected_val) if actual_val != expected_val => {
                diffs.push(format!(
                    "  Field '{}': expected '{}', got '{}'",
                    path, expected_val, actual_val
                ));
            }
            _ => {} // Values match
        }
    }

    // =================== CHANNEL EVENT HELPERS ===================

    /// Creates test values for channel events with unique deterministic values
    pub fn create_channel_test_values(
        suffix: u32,
    ) -> (ConnectionId, ChannelId, PortId, ChannelId, PortId) {
        (
            ConnectionId(suffix + 1000),
            ChannelId(suffix + 2000),
            PortId(Bytes::from(format!("port-{}", suffix))),
            ChannelId(suffix + 3000),
            PortId(Bytes::from(format!("counterpart-{}", suffix))),
        )
    }

    // =================== CONNECTION EVENT HELPERS ===================

    /// Creates test values for connection events with unique deterministic values
    pub fn create_connection_test_values(
        suffix: u32,
    ) -> (ConnectionId, ClientId, ClientId, ConnectionId) {
        (
            ConnectionId(suffix + 1000),
            ClientId(suffix + 2000),
            ClientId(suffix + 3000),
            ConnectionId(suffix + 4000),
        )
    }

    // =================== CLIENT EVENT HELPERS ===================

    /// Creates test values for client events with unique deterministic values
    pub fn create_client_test_values(suffix: u32) -> (ClientId, ClientType, CanonicalChainId) {
        (
            ClientId(suffix + 1000),
            ClientType(format!("client-type-{}", suffix)),
            CanonicalChainId(format!("chain-{}", suffix)),
        )
    }

    // =================== PACKET EVENT HELPERS ===================

    /// Creates test values for packet events with unique deterministic values
    pub fn create_packet_test_values(
        suffix: u32,
    ) -> (ChannelId, PacketHash, Maker, MakerMsg, Acknowledgement) {
        (
            ChannelId(suffix + 1000),
            PacketHash(Bytes::from(format!("packet-hash-{}", suffix))),
            Maker(Bytes::from(format!("maker-{}", suffix))),
            MakerMsg(Bytes::from(format!("maker-msg-{}", suffix))),
            Acknowledgement(Bytes::from(format!("ack-{}", suffix))),
        )
    }

    /// Creates test values for packet send events
    pub fn create_packet_send_test_values(
        suffix: u32,
    ) -> (
        ChannelId,
        PacketHash,
        ChannelId,
        ChannelId,
        BlockHeight,
        TimeoutTimestamp,
        PacketData,
    ) {
        (
            ChannelId(suffix + 1000),
            PacketHash(Bytes::from(format!("packet-hash-{}", suffix))),
            ChannelId(suffix + 2000),
            ChannelId(suffix + 3000),
            BlockHeight(10000 + suffix as u64),
            TimeoutTimestamp(suffix as u64 + 5000000),
            PacketData(Bytes::from(format!("packet-data-{}", suffix))),
        )
    }

    // =================== UCS EVENT HELPERS ===================

    /// Creates test values for token bucket events
    pub fn create_token_bucket_test_values(suffix: u32) -> (Denom, Capacity, RefillRate) {
        (
            Denom(Bytes::from(format!("denom-{}", suffix))),
            Capacity((suffix as u128 + 1000000).try_into().unwrap()),
            RefillRate((suffix as u128 + 100).try_into().unwrap()),
        )
    }

    /// Creates test values for wallet mutation events
    pub fn create_wallet_mutation_test_values(
        suffix: u32,
    ) -> (
        ContractAddress,
        WalletAddress,
        MutationAmount,
        MutationDirection,
    ) {
        (
            ContractAddress(Bytes::from(format!("contract-{}", suffix))),
            WalletAddress(Bytes::from(format!("wallet-{}", suffix))),
            MutationAmount(suffix as u128 + 500),
            MutationDirection::In,
        )
    }

    /// Creates test values for bond events
    pub fn create_bond_test_values(
        suffix: u32,
    ) -> (
        BondInAmount,
        BondMintAmount,
        BondMintToAddress,
        BondSenderAddress,
    ) {
        (
            BondInAmount(suffix as u128 + 500),
            BondMintAmount(suffix as u128 + 1000),
            BondMintToAddress(Bytes::from(format!("bond-mint-to-{suffix}"))),
            BondSenderAddress(Bytes::from(format!("bond-sender-{suffix}"))),
        )
    }

    /// Creates test values for unbond events
    pub fn create_unbond_test_values(
        suffix: u32,
    ) -> (UnbondAmount, Batch, UnbondIsNewRequest, UnbondStakerAddress) {
        (
            UnbondAmount(suffix as u128 + 500),
            Batch(suffix as u64 + 1000),
            UnbondIsNewRequest(true),
            UnbondStakerAddress(Bytes::from(format!("unbond-staker-{suffix}"))),
        )
    }
}
