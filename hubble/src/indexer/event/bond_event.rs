use serde::{Deserialize, Serialize};

use crate::indexer::event::{
    header::Header,
    types::{BondInAmount, BondMintAmount, BondMintToAddress, BondSenderAddress},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BondEvent {
    #[serde(flatten)]
    pub header: Header,
    pub in_amount: BondInAmount,
    pub mint_amount: BondMintAmount,
    pub mint_to_address: BondMintToAddress,
    pub sender: BondSenderAddress,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::event::test_utils::test_helpers::{
        create_bond_test_values, create_test_header, test_json_format, test_roundtrip_serialization,
    };

    /// Creates a test BondEvent with predictable values
    fn create_test_event(suffix: u32) -> BondEvent {
        let header = create_test_header(suffix);
        let (in_amount, mint_amount, mint_to_address, sender) = create_bond_test_values(42);

        BondEvent {
            header,
            in_amount,
            mint_amount,
            mint_to_address,
            sender,
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
            "event_index": "42",
            "height": "10042",
            "in_amount": "542",
            "message_index": "542",
            "mint_amount": "1042",
            "mint_to_address": "0x626f6e642d6d696e742d746f2d3432",
            "sender": "0x626f6e642d73656e6465722d3432",
            "timestamp": "2020-09-13T12:27:22Z",
            "transaction_event_index": "242",
            "transaction_hash": "0x54585f484153485f3432",
            "transaction_index": "142",
            "universal_chain_id": "test-chain-42"
        }"#;

        test_json_format(&event, expected_json);
    }
}
