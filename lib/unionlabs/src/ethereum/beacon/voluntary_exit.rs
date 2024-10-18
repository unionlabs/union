use macros::model;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#voluntaryexit>
#[model]
#[cfg_attr(feature = "ssz", derive(::ssz::Ssz))]
pub struct VoluntaryExit {
    /// Earliest epoch when voluntary exit can be processed
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub epoch: u64,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub validator_index: u64,
}
