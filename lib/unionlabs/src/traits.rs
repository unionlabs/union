use core::fmt::Debug;

use serde::{Deserialize, Serialize};

/// [`Serialize`] and [`Deserialize`] only as exactly [`Self::EXPECTING`].
pub trait FromStrExact: Default + Sized {
    const EXPECTING: &'static str;
}

pub mod from_str_exact {
    use serde::{de, Deserialize, Deserializer};

    use crate::traits::FromStrExact;

    pub fn serialize<S, T: FromStrExact>(_: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(T::EXPECTING)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStrExact,
    {
        let s = <&str>::deserialize(deserializer)?;
        if s == T::EXPECTING {
            Ok(T::default())
        } else {
            Err(de::Error::invalid_value(
                de::Unexpected::Str(s),
                &T::EXPECTING,
            ))
        }
    }
}

/// Trait alias for traits commonly used together throughout this crate.
pub trait Member = Debug
    + Clone
    + PartialEq
    + Serialize
    + for<'de> Deserialize<'de>
    + Send
    + Sync
    + Unpin
    + 'static;
