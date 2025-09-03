//! # `depolama`
//!
//! `depolama` is a simple storage interface for cosmwasm contracts.
//!
//! ```rust
//! use cosmwasm_std::{StdError, StdResult, testing::MockStorage};
//! use depolama::{Bytes, Prefix, Store, KeyCodec, ValueCodec, StorageExt};
//!
//! enum ExampleStore {}
//!
//! impl Store for ExampleStore {
//!     const PREFIX: Prefix = Prefix::new(b"store");
//!
//!     type Key = u64;
//!
//!     type Value = u64;
//! }
//!
//! impl KeyCodec<u64> for ExampleStore {
//!     fn encode_key(key: &u64) -> Bytes {
//!         key.to_be_bytes().into()
//!     }
//!
//!     fn decode_key(raw: &Bytes) -> StdResult<u64> {
//!         raw.try_into().map(u64::from_be_bytes).map_err(|_| {
//!             StdError::generic_err(format!(
//!                 "invalid key: expected 8 bytes, found {}: {raw}",
//!                 raw.len()
//!             ))
//!         })
//!     }
//! }
//!
//! impl ValueCodec<u64> for ExampleStore {
//!     fn encode_value(value: &u64) -> Bytes {
//!         value.to_be_bytes().into()
//!     }
//!
//!     fn decode_value(raw: &Bytes) -> StdResult<u64> {
//!         raw.try_into().map(u64::from_be_bytes).map_err(|_| {
//!             StdError::generic_err(format!(
//!                 "invalid value: expected 8 bytes, found {}: {raw}",
//!                 raw.len()
//!             ))
//!         })
//!     }
//! }
//!
//! let mut storage = MockStorage::new();
//!
//! storage.write::<ExampleStore>(&1, &100);
//! storage.write::<ExampleStore>(&2, &200);
//!
//! assert_eq!(storage.read::<ExampleStore>(&2).unwrap(), 200);
//!
//! assert!(storage.read::<ExampleStore>(&3).is_err());
//!
//! assert!(storage.maybe_read::<ExampleStore>(&3).unwrap().is_none());
//! ```

#![warn(clippy::pedantic, missing_docs)]

use cosmwasm_std::{
    to_json_binary, Addr, Empty, OverflowError, OverflowOperation, Querier, QueryRequest, StdError,
    StdResult, Storage, WasmQuery,
};
use num_traits::{CheckedAdd, One};
#[doc(no_inline)]
pub use unionlabs_primitives::Bytes;

pub use crate::{
    key::{raw_key, KeyCodec},
    value::ValueCodec,
};

/// Storage keys.
pub mod key;
/// Storage values.
pub mod value;

#[cfg(test)]
mod tests;

/// A representation of storage in a cosmwasm contract.
///
/// This is a very simple interface, designed to provide more low-level control over the storage of
/// a cosmwasm contract. Additionally, the key and value codecs are defined directly on this
/// interface, as opposed to on the key/ value types themselves, allowing storage implementors to
/// more easily use external types without relying on direct integration from this crate.
pub trait Store: KeyCodec<Self::Key> + ValueCodec<Self::Value> {
    /// The prefix for this store. See [`Prefix`] for more information.
    const PREFIX: Prefix;

    /// The key used to index this store.
    type Key;

    /// The value stored in this store.
    type Value;
}

/// A storage prefix for a [`Store`] implementation.
///
/// A prefix is arbitrary, potentially empty, non-zero bytes.
pub struct Prefix(PrefixInner);

enum PrefixInner {
    Unprefixed,
    Prefix(&'static [u8]),
}

impl Prefix {
    /// The separator byte used to separate the store prefix and the keys.
    ///
    /// Since [`Prefix`] ensures that the prefix value does not contain this byte, it is guaranteed
    /// that there is no overlap between stores with different prefixes.
    pub const SEPARATOR: u8 = 0x00;

    /// Create a new [`Prefix`].
    ///
    /// # Panics
    ///
    /// This function will panic if `prefix` contains any zero bytes.
    #[must_use = "constructing a `Prefix` has no effect"]
    pub const fn new(prefix: &'static [u8]) -> Self {
        let mut i = 0;

        while i < prefix.len() {
            assert!(
                prefix[i] != Prefix::SEPARATOR,
                "prefix cannot contain the prefix separator byte 0x00"
            );

            i += 1;
        }

        Self(PrefixInner::Prefix(prefix))
    }

    /// Construct a new un-prefixed store prefix.
    ///
    /// If a store's prefix is un-prefixed, no `0x00` byte will be written in the key. This is in
    /// contrast to an *empty* prefix (`Prefix::new(b"")`), where the key will be `[0x00]`.
    ///
    /// This can be useful when dealing with nested stores, or if you want a more strongly-typed
    /// interface to the root store of a contract.
    ///
    /// # Warning
    ///
    /// Un-prefixed stores run the risk of colliding with other (potentially prefixed) stores. Be
    /// sure to use this with caution!
    ///
    /// ```rust
    /// # use depolama::Prefix;
    /// let empty_prefix = Prefix::new(b"").iter_with_separator().copied().collect::<Vec<_>>();
    /// let no_prefix = Prefix::unprefixed().iter_with_separator().copied().collect::<Vec<_>>();
    ///
    /// assert_eq!(empty_prefix, [Prefix::SEPARATOR]);
    /// assert!(no_prefix.is_empty());
    /// ```
    #[must_use = "constructing a `Prefix` has no effect"]
    pub const fn unprefixed() -> Self {
        Self(PrefixInner::Unprefixed)
    }

    /// Returns the length of the prefix, not including the prefix separator byte.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use depolama::Prefix;
    /// assert_eq!(
    ///     b"abc".len(),
    ///     Prefix::new(b"abc").len()
    /// )
    /// ```
    #[must_use = "reading the length has no effect"]
    #[allow(clippy::len_without_is_empty)]
    pub const fn len(&self) -> usize {
        match self.0 {
            PrefixInner::Unprefixed => 0,
            PrefixInner::Prefix(prefix) => prefix.len(),
        }
    }

    /// Return an iterator over the prefix bytes and the separator byte (if any).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use depolama::Prefix;
    /// assert_eq!(
    ///     b"abc\0",
    ///     &*Prefix::new(b"abc")
    ///         .iter_with_separator()
    ///         .copied()
    ///         .collect::<Vec<_>>()
    /// );
    ///
    /// // for unprefixed stores, this will return an empty iterator:
    /// assert_eq!(
    ///     b"",
    ///     &*Prefix::unprefixed()
    ///         .iter_with_separator()
    ///         .copied()
    ///         .collect::<Vec<_>>()
    /// );
    /// ```
    pub fn iter_with_separator(&self) -> impl Iterator<Item = &u8> {
        match self.0 {
            // chain is required to keep the types the same for both branches
            PrefixInner::Unprefixed => [].iter().chain(&[]),
            PrefixInner::Prefix(prefix) => prefix.iter().chain(&[Self::SEPARATOR]),
        }
    }
}

/// Extension trait for [`cosmwasm_std::Storage`] implementations to work with [`Store`]s.
pub trait StorageExt {
    /// Read a value from the store.
    ///
    /// # Errors
    ///
    /// This will return an error if the value is not found or cannot be decoded.
    fn read<S: Store>(&self, k: &S::Key) -> StdResult<S::Value>;

    /// Read the value from the item store.
    ///
    /// # Errors
    ///
    /// This will return an error if the value has not been set or cannot be decoded.
    #[inline]
    fn read_item<S: Store<Key = ()>>(&self) -> StdResult<S::Value> {
        self.read::<S>(&())
    }

    /// Read a value from the store, returning `None` if the value is not found.
    ///
    /// # Errors
    ///
    /// This will return an error if the value cannot be decoded.
    fn maybe_read<S: Store>(&self, k: &S::Key) -> StdResult<Option<S::Value>>;

    /// Read the value from the item store, returning `None` if the value has not been set.
    ///
    /// # Errors
    ///
    /// This will return an error if the value cannot be decoded.
    #[inline]
    fn maybe_read_item<S: Store<Key = ()>>(&self) -> StdResult<Option<S::Value>> {
        self.maybe_read::<S>(&())
    }

    /// Read a value from the store, run the provided closure on the result, and then store the new
    /// value.
    ///
    /// If the value does not exist yet in the store, `f` will be called with `None`, otherwise it
    /// will be called with `Some(S::Value)`.
    ///
    /// # Errors
    ///
    /// This will return an error if the value cannot be decoded.
    #[inline]
    fn upsert<S: Store, E: From<StdError>>(
        &mut self,
        k: &S::Key,
        f: impl FnOnce(Option<S::Value>) -> Result<S::Value, E>,
    ) -> Result<S::Value, E> {
        let value = self.maybe_read::<S>(k)?;
        let v = f(value)?;
        self.write::<S>(k, &v);
        Ok(v)
    }

    /// Read the value from the store, run the provided closure on the result, and then store
    /// the new value.
    ///
    /// If the value has not yet been set in the item store, `f` will be called with `None`,
    /// otherwise it will be called with `Some(S::Value)`.
    ///
    /// # Errors
    ///
    /// This will return an error if the value is not found or cannot be decoded.
    #[inline]
    fn upsert_item<S: Store<Key = ()>, E: From<StdError>>(
        &mut self,
        f: impl FnOnce(Option<S::Value>) -> Result<S::Value, E>,
    ) -> Result<S::Value, E> {
        self.upsert::<S, E>(&(), f)
    }

    /// Read the value from the item store, run the provided closure on the result, and then store
    /// the new value.
    ///
    /// # Errors
    ///
    /// This will return an error if the value is not found or cannot be decoded.
    #[inline]
    fn update<S: Store, E: From<StdError>, R>(
        &mut self,
        k: &S::Key,
        f: impl FnOnce(&mut S::Value) -> Result<R, E>,
    ) -> Result<R, E> {
        let mut v = self.read::<S>(k)?;
        let r = f(&mut v)?;
        self.write::<S>(k, &v);
        Ok(r)
    }

    /// Read the value from the item store, run the provided closure on the result, and then store
    /// the new value.
    ///
    /// # Errors
    ///
    /// This will return an error if the value is not found or cannot be decoded.
    #[inline]
    fn update_item<S: Store<Key = ()>, E: From<StdError>, R>(
        &mut self,
        f: impl FnOnce(&mut S::Value) -> Result<R, E>,
    ) -> Result<R, E> {
        let mut v = self.read_item::<S>()?;
        let r = f(&mut v)?;
        self.write_item::<S>(&v);
        Ok(r)
    }

    /// Write a value to the store.
    fn write<S: Store>(&mut self, k: &S::Key, v: &S::Value);

    /// Write the value to the item store.
    #[inline]
    fn write_item<S: Store<Key = ()>>(&mut self, v: &S::Value) {
        self.write::<S>(&(), v);
    }

    /// Increment the value under the item store.
    ///
    /// # Errors
    ///
    /// This will return an error if the value cannot be decoded, or if the value would overflow.
    #[inline]
    fn increment_item<S: Store<Key = (), Value: One + CheckedAdd>>(
        &mut self,
    ) -> Result<S::Value, StdError> {
        let value = self.read::<S>(&())?;
        let v = value
            .checked_add(&S::Value::one())
            .ok_or(StdError::overflow(OverflowError::new(
                OverflowOperation::Add,
            )))?;
        self.write::<S>(&(), &v);
        Ok(v)
    }

    /// Delete a value from the store.
    fn delete<S: Store>(&mut self, k: &S::Key);

    /// Delete the value from the item store.
    #[inline]
    fn delete_item<S: Store<Key = ()>>(&mut self) {
        self.delete::<S>(&());
    }

    /// Iterate over all of the (key, value) pairs in the store.
    ///
    /// # Errors
    ///
    /// Each produced item will return an error if either the key or value cannot be decoded.
    #[cfg(feature = "iterator")]
    fn iter<S: Store>(
        &self,
        order: cosmwasm_std::Order,
    ) -> impl Iterator<Item = StdResult<(S::Key, S::Value)>>;

    /// Iterate over all of the (key, value) pairs in the store within the bounds.
    ///
    /// # Errors
    ///
    /// Each produced item will return an error if either the key or value cannot be decoded.
    #[cfg(feature = "iterator")]
    fn iter_range<S: Store>(
        &self,
        order: cosmwasm_std::Order,
        bounds: impl std::ops::RangeBounds<S::Key>,
    ) -> impl Iterator<Item = StdResult<(S::Key, S::Value)>>;
}

impl<T: Storage> StorageExt for T {
    fn read<S: Store>(&self, k: &S::Key) -> StdResult<S::Value> {
        (self as &dyn Storage).read::<S>(k)
    }

    fn maybe_read<S: Store>(&self, k: &S::Key) -> StdResult<Option<S::Value>> {
        (self as &dyn Storage).maybe_read::<S>(k)
    }

    fn write<S: Store>(&mut self, k: &S::Key, v: &S::Value) {
        (self as &mut dyn Storage).write::<S>(k, v);
    }

    fn delete<S: Store>(&mut self, k: &S::Key) {
        (self as &mut dyn Storage).delete::<S>(k);
    }

    #[cfg(feature = "iterator")]
    fn iter<S: Store>(
        &self,
        order: cosmwasm_std::Order,
    ) -> impl Iterator<Item = StdResult<(S::Key, S::Value)>> {
        (self as &dyn Storage).iter::<S>(order)
    }

    #[cfg(feature = "iterator")]
    fn iter_range<S: Store>(
        &self,
        order: cosmwasm_std::Order,
        bounds: impl std::ops::RangeBounds<S::Key>,
    ) -> impl Iterator<Item = StdResult<(S::Key, S::Value)>> {
        (self as &dyn Storage).iter_range::<S>(order, bounds)
    }
}

impl StorageExt for dyn Storage + '_ {
    #[inline]
    fn read<S: Store>(&self, k: &S::Key) -> StdResult<S::Value> {
        self.maybe_read::<S>(k)?
            .ok_or_else(|| StdError::generic_err(format!("key {} not present", S::encode_key(k))))
    }

    #[inline]
    fn maybe_read<S: Store>(&self, k: &S::Key) -> StdResult<Option<S::Value>> {
        match self.get(&raw_key::<S>(k)) {
            Some(v) => S::decode_value(&Bytes::new(v)).map(Some),
            None => Ok(None),
        }
    }

    #[inline]
    fn write<S: Store>(&mut self, k: &S::Key, v: &S::Value) {
        self.set(&raw_key::<S>(k), &S::encode_value(v));
    }

    #[inline]
    fn delete<S: Store>(&mut self, k: &S::Key) {
        self.remove(&raw_key::<S>(k));
    }

    #[cfg(feature = "iterator")]
    #[inline]
    fn iter<S: Store>(
        &self,
        order: cosmwasm_std::Order,
    ) -> impl Iterator<Item = StdResult<(S::Key, S::Value)>> {
        self.iter_range::<S>(order, ..)
    }

    #[cfg(feature = "iterator")]
    #[inline]
    fn iter_range<S: Store>(
        &self,
        order: cosmwasm_std::Order,
        bounds: impl std::ops::RangeBounds<S::Key>,
    ) -> impl Iterator<Item = StdResult<(S::Key, S::Value)>> {
        use std::ops::Bound;

        let key_plus_one = |key| {
            let raw_key = raw_key::<S>(key);

            let mut raw_key = raw_key.into_vec();

            'block: {
                for byte in raw_key.iter_mut().rev() {
                    if *byte == u8::MAX {
                        continue;
                    }

                    *byte += 1;
                    break 'block;
                }

                // [] < [0]
                raw_key.push(0);
            }

            Some(raw_key.into())
        };

        let from = match bounds.start_bound() {
            // start is inclusive at the db layer
            Bound::Included(key) => Some(raw_key::<S>(key)),
            // to exclude the end, construct a key > raw_key
            Bound::Excluded(key) => key_plus_one(key),
            Bound::Unbounded => None,
        };

        let to = match bounds.end_bound() {
            // to include the end, construct a key > raw_key
            Bound::Included(key) => key_plus_one(key),
            // end is exclusive at the db layer
            Bound::Excluded(key) => Some(raw_key::<S>(key)),
            Bound::Unbounded => None,
        };

        dbg!(&from, &to);

        self.range(from.as_deref(), to.as_deref(), order)
            .map(|(k, v)| {
                Ok((
                    S::decode_key(&Bytes::new(k[(S::PREFIX.len() + 1)..].to_vec()))?,
                    S::decode_value(&Bytes::new(v))?,
                ))
            })
    }
}

/// Extension trait for [`cosmwasm_std::Querier`] implementations to work with [`Store`]s.
pub trait QuerierExt {
    /// Read a value from the store of another contract.
    ///
    /// # Errors
    ///
    /// This will error if the cross contract call fails, or if the value cannot be decoded.
    fn read<S: Store>(&self, addr: &Addr, key: &S::Key) -> StdResult<S::Value>;

    /// Read a value from the item store of another contract.
    ///
    /// # Errors
    ///
    /// This will error if the cross contract call fails, or if the value cannot be decoded.
    fn read_item<S: Store<Key = ()>>(&self, addr: &Addr) -> StdResult<S::Value>;
}

impl QuerierExt for dyn Querier + '_ {
    #[inline]
    fn read<S: Store>(&self, addr: &Addr, key: &S::Key) -> StdResult<S::Value> {
        let raw_value = self
            .raw_query(
                &to_json_binary(&QueryRequest::<Empty>::Wasm(WasmQuery::Raw {
                    contract_addr: addr.into(),
                    key: raw_key::<S>(key).into_vec().into(),
                }))
                .expect("serialization is infallible; qed;"),
            )
            .into_result()
            .map_err(|e| StdError::generic_err(e.to_string()))?
            .into_result()
            .map_err(StdError::generic_err)?;

        S::decode_value(&<Vec<u8>>::from(raw_value).into())
    }

    #[inline]
    fn read_item<S: Store<Key = ()>>(&self, addr: &Addr) -> StdResult<S::Value> {
        self.read::<S>(addr, &())
    }
}

/// A raw, un-prefixed store implementation.
///
/// This is equivalent to directly reading `deps.storage`, but with a slightly nicer api and
/// composes nicely with the apis in this crate.
pub enum RawStore {}
impl Store for RawStore {
    const PREFIX: Prefix = Prefix::unprefixed();

    type Key = Bytes;

    type Value = Bytes;
}
impl KeyCodec<Bytes> for RawStore {
    fn encode_key(key: &Bytes) -> Bytes {
        key.clone()
    }

    fn decode_key(raw: &Bytes) -> StdResult<Bytes> {
        Ok(raw.clone())
    }
}
impl ValueCodec<Bytes> for RawStore {
    fn encode_value(value: &Bytes) -> Bytes {
        value.clone()
    }

    fn decode_value(raw: &Bytes) -> StdResult<Bytes> {
        Ok(raw.clone())
    }
}
