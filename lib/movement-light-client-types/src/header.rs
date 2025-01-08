use ethereum_light_client_types::{AccountProof, StorageProof};
use unionlabs::{
    aptos::{state_proof::StateProof, transaction_proof::TransactionInfoWithProof},
    ibc::core::client::height::Height,
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Header {
    pub l1_height: u64,
    pub trusted_height: Height,
    pub state_proof: StateProof,
    pub tx_index: u64,
    pub tx_proof: TransactionInfoWithProof,
    /// Proof that the hash of the `StateProof` is committed to L1
    pub state_proof_hash_proof: StorageProof,
    /// Proof of state of the settlement contract on L1
    pub settlement_contract_proof: AccountProof,
    pub new_height: u64,
}
