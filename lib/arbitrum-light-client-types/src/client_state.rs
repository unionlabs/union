use unionlabs::{
    bounded::BoundedU32, ibc::core::client::height::Height, id::ClientId, primitives::H160,
    uint::U256,
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClientState {
    pub l1_client_id: ClientId,
    pub chain_id: U256,
    pub l1_latest_slot: u64,
    pub l1_contract_address: H160,
    pub l1_next_node_num_slot: U256,
    pub l1_nodes_slot: U256,
    // this is a u32 because protobuf sucks
    pub l1_next_node_num_slot_offset_bytes: BoundedU32<0, 24>,
    // TODO: Rename this in the protos
    pub l1_nodes_confirm_data_offset: U256,
    pub frozen_height: Height,
    pub l2_ibc_contract_address: H160,
}
