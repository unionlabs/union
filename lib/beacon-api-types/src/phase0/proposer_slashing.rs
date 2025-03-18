use crate::phase0::SignedBeaconBlockHeader;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct ProposerSlashing {
    pub signed_header_1: SignedBeaconBlockHeader,
    pub signed_header_2: SignedBeaconBlockHeader,
}
