use unionlabs::{
    ibc::core::client::height::Height,
    primitives::{H160, U256},
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClientState {
    pub chain_id: u64,
    pub frozen_height: Height,
    pub ibc_contract_address: H160,
    // TODO: This should be ClientId
    pub l1_client_id: u32,
    pub l2_committed_batches_slot: U256,
    pub l2_contract_address: H160,
    pub l2_finalized_state_roots_slot: U256,
    pub latest_batch_index_slot: U256,
    pub latest_slot: u64,
}
