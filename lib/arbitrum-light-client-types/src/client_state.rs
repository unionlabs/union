use ibc_union_spec::ClientId;
use unionlabs::{
    bounded::BoundedU8,
    ibc::core::client::height::Height,
    primitives::{H160, U256},
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum ClientState {
    V1(ClientStateV1),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ClientStateV1 {
    pub chain_id: U256,

    /// Latest height of the L2
    pub latest_height: u64,

    /// Client id of the client tracking the L1 that the chain this client tracks settles on
    pub l1_client_id: ClientId,

    /// Rollup contract on the L1
    pub l1_contract_address: H160,

    // various config params
    pub l1_next_node_num_slot: U256,
    pub l1_nodes_slot: U256,
    pub l1_next_node_num_slot_offset_bytes: BoundedU8<0, 24>,
    pub l1_nodes_confirm_data_offset: U256,

    pub frozen_height: Height,

    pub ibc_contract_address: H160,
}
