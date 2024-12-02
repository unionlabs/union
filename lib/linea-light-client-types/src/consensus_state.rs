use unionlabs::hash::H256;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConsensusState {
    pub ibc_storage_root: H256,
    pub timestamp: u64,
}
