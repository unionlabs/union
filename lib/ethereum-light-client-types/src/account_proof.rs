use unionlabs::primitives::H256;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AccountProof {
    pub storage_root: H256,
    pub proof: Vec<Vec<u8>>,
}
