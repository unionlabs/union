use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz), ssz(transparent))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Slot(u64);

impl Slot {
    pub const fn new(slot: u64) -> Self {
        Self(slot)
    }

    pub const fn get(&self) -> u64 {
        self.0
    }
}

impl Add<Self> for Slot {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.0 + rhs.0)
    }
}

impl Sub<Self> for Slot {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.0 - rhs.0)
    }
}

impl Mul<Self> for Slot {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.0 * rhs.0)
    }
}

impl Div<Self> for Slot {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.0 / rhs.0)
    }
}

impl Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Slot {
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
impl<'de> serde::Deserialize<'de> for Slot {
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
