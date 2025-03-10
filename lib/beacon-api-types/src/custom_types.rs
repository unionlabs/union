use hex_literal::hex;
use unionlabs::primitives::FixedBytes;

#[derive(Debug, Clone, PartialEq, Eq, Default, Copy)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz), ssz(transparent))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Version(pub FixedBytes<4>);

#[derive(Debug, Clone, PartialEq, Eq, Default, Copy)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz), ssz(transparent))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct DomainType(pub FixedBytes<4>);

#[derive(Debug, Clone, PartialEq, Eq, Default, Copy)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz), ssz(transparent))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ForkDigest(pub FixedBytes<4>);

#[derive(Debug, Clone, PartialEq, Eq, Default, Copy)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz), ssz(transparent))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Domain(pub FixedBytes<32>);

/// <https://github.com/ethereum/consensus-specs/blob/087e7378b44f327cdad4549304fc308613b780c3/specs/phase0/beacon-chain.md#domain-types>
#[rustfmt::skip]
impl DomainType {
    pub const BEACON_PROPOSER: Self                = Self(FixedBytes::new(hex!("00000000")));
    pub const BEACON_ATTESTER: Self                = Self(FixedBytes::new(hex!("01000000")));
    pub const RANDAO: Self                         = Self(FixedBytes::new(hex!("02000000")));
    pub const DEPOSIT: Self                        = Self(FixedBytes::new(hex!("03000000")));
    pub const VOLUNTARY_EXIT: Self                 = Self(FixedBytes::new(hex!("04000000")));
    pub const SELECTION_PROOF: Self                = Self(FixedBytes::new(hex!("05000000")));
    pub const AGGREGATE_AND_PROOF: Self            = Self(FixedBytes::new(hex!("06000000")));
    pub const SYNC_COMMITTEE: Self                 = Self(FixedBytes::new(hex!("07000000")));
    pub const SYNC_COMMITTEE_SELECTION_PROOF: Self = Self(FixedBytes::new(hex!("08000000")));
    pub const CONTRIBUTION_AND_PROOF: Self         = Self(FixedBytes::new(hex!("09000000")));
    pub const BLS_TO_EXECUTION_CHANGE: Self        = Self(FixedBytes::new(hex!("0A000000")));
    pub const APPLICATION_MASK: Self               = Self(FixedBytes::new(hex!("00000001")));
}
