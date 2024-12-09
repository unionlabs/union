use unionlabs::{hash::H160, uint::U256};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClientState {
    pub comet: tendermint_light_client_types::ClientState,
    pub execution_chain_id: U256,
    // latest execution height
    pub execution_latest_height: u64,
    /// the ibc contract on the counterparty chain that contains the ICS23 commitments
    pub ibc_contract_address: H160,
}
