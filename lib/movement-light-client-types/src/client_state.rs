use unionlabs::{aptos::account::AccountAddress, hash::H160, ibc::core::client::height::Height};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClientState {
    pub chain_id: String,
    pub l1_client_id: u32,
    pub l1_contract_address: H160,
    pub l2_contract_address: AccountAddress,
    pub table_handle: AccountAddress,
    pub frozen_height: Height,
    pub latest_block_num: u64,
}
