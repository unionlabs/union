use core::{fmt, num::NonZeroU32};

use unionlabs::{errors::UnknownEnumVariant, primitives::U256};

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

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum Status {
    Active = 1,
    Expired = 2,
    Frozen = 3,
}

impl From<Status> for U256 {
    fn from(value: Status) -> Self {
        U256::from((value as u8) as u64)
    }
}

impl TryFrom<U256> for Status {
    type Error = UnknownEnumVariant<U256>;

    fn try_from(value: U256) -> Result<Self, Self::Error> {
        match u64::try_from(value).map_err(|()| UnknownEnumVariant(value))? {
            1 => Ok(Status::Active),
            2 => Ok(Status::Expired),
            3 => Ok(Status::Frozen),
            unknown => Err(UnknownEnumVariant(unknown.into())),
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Status::Active => "active",
            Status::Expired => "expired",
            Status::Frozen => "frozen",
        })
    }
}
