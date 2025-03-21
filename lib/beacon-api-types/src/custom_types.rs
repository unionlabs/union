use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
};

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

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#domain-types>
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

macro_rules! u64_newtype {
    ($T:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[cfg_attr(feature = "ssz", derive(ssz::Ssz), ssz(transparent))]
        #[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
        pub struct $T(u64);

        impl $T {
            pub const fn new(slot: u64) -> Self {
                Self(slot)
            }

            pub const fn get(&self) -> u64 {
                self.0
            }

            pub const fn is_zero(&self) -> bool {
                self.0 == 0
            }
        }

        impl Add<Self> for $T {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self::new(self.0 + rhs.0)
            }
        }

        impl Add<u64> for $T {
            type Output = Self;

            fn add(self, rhs: u64) -> Self::Output {
                Self::new(self.0 + rhs)
            }
        }

        impl Sub<Self> for $T {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self::new(self.0 - rhs.0)
            }
        }

        impl Sub<u64> for $T {
            type Output = Self;

            fn sub(self, rhs: u64) -> Self::Output {
                Self::new(self.0 - rhs)
            }
        }

        impl Mul<Self> for $T {
            type Output = Self;

            fn mul(self, rhs: Self) -> Self::Output {
                Self::new(self.0 * rhs.0)
            }
        }

        impl Mul<u64> for $T {
            type Output = Self;

            fn mul(self, rhs: u64) -> Self::Output {
                Self::new(self.0 * rhs)
            }
        }

        impl Div<Self> for $T {
            type Output = Self;

            fn div(self, rhs: Self) -> Self::Output {
                Self::new(self.0 / rhs.0)
            }
        }

        impl Div<u64> for $T {
            type Output = Self;

            fn div(self, rhs: u64) -> Self::Output {
                Self::new(self.0 / rhs)
            }
        }

        impl Display for $T {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        #[cfg(feature = "serde")]
        impl serde::Serialize for $T {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                if serializer.is_human_readable() {
                    serializer.collect_str(&self.0)
                } else {
                    self.0.serialize(serializer)
                }
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> serde::Deserialize<'de> for $T {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                if deserializer.is_human_readable() {
                    String::deserialize(deserializer).and_then(|s| {
                        s.parse()
                            // TODO fix error situation
                            // FromStr::Err has no bounds
                            .map_err(|_| serde::de::Error::custom("failure to parse string data"))
                            .map(Self)
                    })
                } else {
                    u64::deserialize(deserializer).map(Self)
                }
            }
        }
    };
}

u64_newtype!(Slot);
u64_newtype!(Period);
u64_newtype!(Epoch);
u64_newtype!(ValidatorIndex);
u64_newtype!(Gwei);
u64_newtype!(Gas);
u64_newtype!(CommitteeIndex);
u64_newtype!(WithdrawalIndex);
