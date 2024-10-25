#[derive(Debug, Clone, PartialEq, ssz::Ssz)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VoluntaryExit {
    /// Earliest epoch when voluntary exit can be processed
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub epoch: u64,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub validator_index: u64,
}
