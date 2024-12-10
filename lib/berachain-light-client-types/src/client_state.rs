use unionlabs::{hash::H160, uint::U256};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClientState {
    /// consensus client
    pub l1_client_id: u32,
    /// execution chain id
    pub chain_id: U256,
    /// execution height
    pub latest_height: u64,
    /// the ibc contract on the counterparty chain that contains the ICS23 commitments
    pub ibc_contract_address: H160,
}
