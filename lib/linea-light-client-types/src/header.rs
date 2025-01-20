use ethereum_light_client_types::{AccountProof, StorageProof};
use linea_types::proof::InclusionProof;
use unionlabs::ibc::core::client::height::Height;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Header {
    pub l1_height: Height,
    pub l1_rollup_contract_proof: AccountProof,
    pub l2_timestamp_proof: StorageProof,
    pub l2_block_number_proof: StorageProof,
    pub l2_state_root_proof: StorageProof,
    pub l2_ibc_contract_proof: InclusionProof,
}
