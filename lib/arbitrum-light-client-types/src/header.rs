use ethereum_light_client_types::{AccountProof, StorageProof};
use unionlabs::ibc::core::client::height::Height;

use crate::L2Header;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Header {
    pub l1_height: Height,
    pub l1_account_proof: AccountProof,
    pub l2_ibc_account_proof: AccountProof,
    pub l1_next_node_num_slot_proof: StorageProof,
    pub l1_nodes_slot_proof: StorageProof,
    pub l2_header: L2Header,
}
