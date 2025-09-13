use serde::{Deserialize, Serialize};

use crate::indexer::event::{
    header::Header,
    types::{ContractAddress, MutationAmount, MutationDirection, WalletAddress},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct WalletMutationEntryEvent {
    #[serde(flatten)]
    pub header: Header,
    pub contract_address_canonical: ContractAddress,
    pub wallet_address_canonical: WalletAddress,
    pub amount: MutationAmount,
    pub direction: MutationDirection,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::event::test_utils::test_helpers::*;

    /// Creates a test WalletMutationEntryEvent with unique deterministic values
    fn create_test_event(suffix: u32) -> WalletMutationEntryEvent {
        let (contract_address_canonical, wallet_address_canonical, amount, direction) =
            create_wallet_mutation_test_values(suffix);

        WalletMutationEntryEvent {
            header: create_test_header(suffix),
            contract_address_canonical,
            wallet_address_canonical,
            amount,
            direction,
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

        // Test that the JSON format matches expectations

        let expected_json = r#"{
  "amount": "542",
  "block_hash": "0x424c4f434b5f484153485f3432",
  "contract_address_canonical": "0x636f6e74726163742d3432",
  "direction": "in",
  "event_index": "42",
  "height": "10042",
  "message_index": "542",
  "timestamp": "2020-09-13T12:27:22Z",
  "transaction_event_index": "242",
  "transaction_hash": "0x54585f484153485f3432",
  "transaction_index": "142",
  "universal_chain_id": "test-chain-42",
  "wallet_address_canonical": "0x77616c6c65742d3432"
}"#;

        test_json_format(&event, expected_json);
    }
}
