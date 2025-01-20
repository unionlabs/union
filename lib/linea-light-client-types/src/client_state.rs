use unionlabs::{
    ibc::core::client::height::Height,
    primitives::{H160, U256},
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClientState {
    pub chain_id: U256,
    // TODO: This should be ClientId
    pub l1_client_id: String,
    pub l1_latest_height: Height,
    pub l1_rollup_contract_address: H160,
    pub l1_rollup_current_l2_timestamp_slot: U256,
    pub l1_rollup_current_l2_block_number_slot: U256,
    pub l1_rollup_l2_state_root_hashes_slot: U256,
    pub l2_ibc_contract_address: H160,
    pub frozen_height: Height,
}
