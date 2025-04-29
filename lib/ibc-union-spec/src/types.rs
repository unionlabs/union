use core::num::NonZeroU32;

use unionlabs::primitives::U256;

pub(crate) mod channel;
pub(crate) mod connection;
pub(crate) mod packet;

macro_rules! id {
    ($T:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
        #[cfg_attr(
            feature = "serde",
            derive(serde::Serialize, serde::Deserialize),
            serde(transparent)
        )]
        #[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
        pub struct $T(#[doc(hidden)] NonZeroU32);

        impl $T {
            pub const fn from_raw(raw: u32) -> Option<$T> {
                match NonZeroU32::new(raw) {
                    Some(id) => Some(Self(id)),
                    None => None,
                }
            }

            pub const fn new(id: NonZeroU32) -> Self {
                Self(id)
            }

            pub const fn get(&self) -> NonZeroU32 {
                self.0
            }

            pub const fn raw(&self) -> u32 {
                self.0.get()
            }

            pub const fn checked_add(&self, rhs: u32) -> Option<Self> {
                match self.get().checked_add(rhs) {
                    Some(id) => Some(Self::new(id)),
                    None => None,
                }
            }
        }

        impl core::fmt::Display for $T {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl core::str::FromStr for $T {
            type Err = <NonZeroU32 as core::str::FromStr>::Err;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                s.parse().map(Self::new)
            }
        }

        impl From<$T> for U256 {
            fn from(id: $T) -> Self {
                id.raw().into()
            }
        }

        impl From<NonZeroU32> for $T {
            fn from(id: NonZeroU32) -> Self {
                <$T>::new(id)
            }
        }

        impl TryFrom<u32> for $T {
            type Error = <NonZeroU32 as TryFrom<u32>>::Error;

            fn try_from(raw: u32) -> Result<Self, Self::Error> {
                raw.try_into().map(Self::new)
            }
        }

        #[macro_export]
        macro_rules! $T {
            ($raw:expr) => {
                <$T>::new(core::num::NonZeroU32::new($raw).unwrap())
            };
        }
    };
}

id!(ClientId);
id!(ConnectionId);
id!(ChannelId);
