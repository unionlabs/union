use unionlabs::{
    aptos::{state_proof::StateProof, transaction_proof::TransactionInfoWithProof},
    ibc::core::client::height::Height,
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Header {
    pub trusted_height: Height,
    pub state_proof: StateProof,
    pub tx_index: u64,
    pub tx_proof: TransactionInfoWithProof,
    pub new_height: u64,
}
