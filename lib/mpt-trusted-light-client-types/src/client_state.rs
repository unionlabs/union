use unionlabs::primitives::{H160, U256};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ClientState {
    pub chain_id: U256,
    pub latest_height: u64,
    pub ibc_contract_address: H160,
}
