use ethereum_light_client_types::{AccountProof, StorageProof};
use unionlabs::ibc::core::client::height::Height;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Header {
    pub l1_height: Height,
    pub l1_account_proof: AccountProof,
    /// This is the finalized state root proof, i.e. the l2 state on the l1
    pub l2_state_root_proof: StorageProof,
    pub last_batch_index_proof: StorageProof,
    pub batch_hash_proof: StorageProof,
    pub l2_ibc_account_proof: AccountProof,
    pub batch_header: Vec<u8>,
}
