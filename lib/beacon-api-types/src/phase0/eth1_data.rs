use unionlabs::primitives::H256;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct Eth1Data {
    pub deposit_root: H256,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub deposit_count: u64,
    pub block_hash: H256,
}
