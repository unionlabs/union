// Copyright (c) The camino Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Serde implementations for `Utf8Path`.
//!
//! The Serde implementations for `Utf8PathBuf` are derived, but `Utf8Path` is an unsized type which
//! the derive impls can't handle. Implement these by hand.

use crate::Utf8Path;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

struct Utf8PathVisitor;

impl<'a> de::Visitor<'a> for Utf8PathVisitor {
    type Value = &'a Utf8Path;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a borrowed path")
    }

    fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v.as_ref())
    }

    fn visit_borrowed_bytes<E>(self, v: &'a [u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        std::str::from_utf8(v)
            .map(AsRef::as_ref)
            .map_err(|_| de::Error::invalid_value(de::Unexpected::Bytes(v), &self))
    }
}

#[cfg(feature = "serde1")]
impl<'de: 'a, 'a> Deserialize<'de> for &'a Utf8Path {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(Utf8PathVisitor)
    }
}

#[cfg(feature = "serde1")]
impl Serialize for Utf8Path {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_str().serialize(serializer)
    }
}
