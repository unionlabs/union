#[derive(Debug, Clone, PartialEq, ssz::Ssz)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SigningData {
    pub object_root: H256,
    pub domain: Domain,
}
