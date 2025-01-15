use unionlabs::{
    aptos::account::AccountAddress, ibc::core::client::height::Height, primitives::H160,
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ClientState {
    pub chain_id: String,
    pub l1_client_id: u32,
    pub l1_contract_address: H160,
    pub l2_contract_address: AccountAddress,
    pub table_handle: AccountAddress,
    pub frozen_height: Height,
    pub latest_block_num: u64,
    pub whitelisted_relayers: Vec<String>,
}
