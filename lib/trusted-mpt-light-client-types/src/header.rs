use consensus_primitives::Timestamp;
use ethereum_light_client_types::AccountProof;
use unionlabs::primitives::H256;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Header {
    pub height: u64,
    pub timestamp: Timestamp,
    pub state_root: H256,
    pub ibc_account_proof: AccountProof,
}
