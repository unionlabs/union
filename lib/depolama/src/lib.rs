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
    to_json_binary, Addr, Empty, Querier, QueryRequest, StdError, StdResult, Storage, WasmQuery,
};
#[doc(no_inline)]
pub use unionlabs_primitives::Bytes;

/// A representation of storage in a cosmwasm contract.
///
/// This is a very simple interface, designed to provide more low-level control over the storage of
/// a cosmwasm contract. Additionally, the key and value codecs are defined directly on this
/// interface, as opposed to on the key/ value types themselves, allowing storage implementors to
/// more easily use external types without relying on direct integration with this crate.
pub trait Store: KeyCodec<Self::Key> + ValueCodec<Self::Value> {
    /// The prefix for this store. See [`Prefix`] for more information.
    const PREFIX: Prefix;

    /// The key used to index this store.
    type Key;

    /// The value stored in this store.
    type Value;
}

/// The key encoding and decoding for a [`Store`].
///
/// This trait is implemented for all stores with `type Key = ()`, enabling "item store" functionality on [`StorageExt`].
pub trait KeyCodec<Key> {
    /// Encode the given key for writing to storage.
    ///
    /// # Implementor's Note
    ///
    /// This function is expected to be isomorphic with [`KeyCodec::decode_key`].
    fn encode_key(key: &Key) -> Bytes;

    /// Decode the key for this store.
    ///
    /// # Errors
    ///
    /// This function is expected to error iff the key cannot be decoded.
    ///
    /// # Implementor's Note
    ///
    /// This function is expected to be isomorphic with [`KeyCodec::encode_key`].
    fn decode_key(raw: &Bytes) -> StdResult<Key>;
}

impl<T: Store<Key = ()>> KeyCodec<()> for T {
    fn encode_key((): &()) -> Bytes {
        [].into()
    }

    fn decode_key(raw: &Bytes) -> StdResult<()> {
        if raw.is_empty() {
            Ok(())
        } else {
            Err(StdError::generic_err(format!(
                "key must be empty, found {raw}"
            )))
        }
    }
}

/// The value encoding and decoding for a [`Store`].
pub trait ValueCodec<Value> {
    /// Encode the given value for writing to storage.
    ///
    /// # Implementor's Note
    ///
    /// This function is expected to be isomorphic with [`ValueCodec::decode_value`].
    fn encode_value(value: &Value) -> Bytes;

    /// Decode the value for this store.
    ///
    /// # Errors
    ///
    /// This function is expected to error iff the key cannot be decoded.
    ///
    /// # Implementor's Note
    ///
    /// This function is expected to be isomorphic with [`ValueCodec::encode_value`].
    fn decode_value(raw: &Bytes) -> StdResult<Value>;
}

/// A storage prefix for a [`Store`] implementation.
///
/// A prefix is arbitrary, potentially empty, non-zero bytes.
pub struct Prefix(&'static [u8]);

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

        Self(prefix)
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
        self.0.len()
    }

    /// Return an iterator over the prefix keys and the seperator byte.
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
    /// )
    /// ```
    pub fn iter_with_separator(&self) -> impl Iterator<Item = &u8> {
        self.0.iter().chain(&[Self::SEPARATOR])
    }
}
/// The raw store prefix for the store.
pub fn raw_key<S: Store>(key: &S::Key) -> Bytes {
    S::PREFIX
        .iter_with_separator()
        .copied()
        .chain(S::encode_key(key))
        .collect()
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

    /// Read the value from the item store, run the provided closure on the result, and then store the new
    /// value.
    ///
    /// If the value has not yet been set in the item store, `f` will be called with `None`, otherwise it
    /// will be called with `Some(S::Value)`.
    ///
    /// # Errors
    ///
    /// This will return an error if the value cannot be decoded.
    fn upsert_item<S: Store<Key = ()>, E: From<StdError>>(
        &mut self,
        f: impl FnOnce(Option<S::Value>) -> Result<S::Value, E>,
    ) -> Result<S::Value, E> {
        self.upsert::<S, E>(&(), f)
    }

    /// Write a value to the store.
    fn write<S: Store>(&mut self, k: &S::Key, v: &S::Value);

    /// Write the value to the item store.
    fn write_item<S: Store<Key = ()>>(&mut self, v: &S::Value) {
        self.write::<S>(&(), v);
    }

    /// Delete a value from the store.
    fn delete<S: Store>(&mut self, k: &S::Key);

    /// Delete the value from the item store.
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
}

impl<'a> StorageExt for dyn Storage + 'a {
    fn read<S: Store>(&self, k: &S::Key) -> StdResult<S::Value> {
        self.maybe_read::<S>(k)?
            .ok_or_else(|| StdError::generic_err(format!("key {} not present", S::encode_key(k))))
    }

    fn maybe_read<S: Store>(&self, k: &S::Key) -> StdResult<Option<S::Value>> {
        match self.get(&raw_key::<S>(k)) {
            Some(v) => S::decode_value(&Bytes::new(v)).map(Some),
            None => Ok(None),
        }
    }

    fn write<S: Store>(&mut self, k: &S::Key, v: &S::Value) {
        self.set(&raw_key::<S>(k), &S::encode_value(v));
    }

    fn delete<S: Store>(&mut self, k: &S::Key) {
        self.remove(&raw_key::<S>(k));
    }

    #[cfg(feature = "iterator")]
    fn iter<S: Store>(
        &self,
        order: cosmwasm_std::Order,
    ) -> impl Iterator<Item = StdResult<(S::Key, S::Value)>> {
        let from = S::PREFIX.iter_with_separator().copied().collect::<Vec<_>>();
        let mut to = from.clone();
        *to.last_mut()
            .expect("length is at least one due to containing separator; qed;") += 1;

        self.range(Some(&from), Some(&to), order).map(|(k, v)| {
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
}

impl QuerierExt for dyn Querier + '_ {
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
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::MockStorage;
    use unionlabs::ByteArrayExt;

    use super::*;

    enum TestStore {}

    impl Store for TestStore {
        const PREFIX: Prefix = Prefix::new(b"test");

        type Key = u64;

        type Value = (u64, u64);
    }

    impl KeyCodec<u64> for TestStore {
        fn encode_key(key: &u64) -> Bytes {
            key.to_be_bytes().into()
        }

        fn decode_key(raw: &Bytes) -> StdResult<u64> {
            raw.try_into().map(u64::from_be_bytes).map_err(|_| {
                StdError::generic_err(format!(
                    "invalid key: expected 8 bytes, found {}: {raw}",
                    raw.len()
                ))
            })
        }
    }

    impl ValueCodec<(u64, u64)> for TestStore {
        fn encode_value(value: &(u64, u64)) -> Bytes {
            [value.0.to_be_bytes(), value.1.to_be_bytes()]
                .into_iter()
                .flatten()
                .collect()
        }

        fn decode_value(raw: &Bytes) -> StdResult<(u64, u64)> {
            raw.try_into()
                .map(|arr: [u8; 16]| {
                    (
                        u64::from_be_bytes(arr.array_slice::<0, 8>()),
                        u64::from_be_bytes(arr.array_slice::<8, 8>()),
                    )
                })
                .map_err(|_| {
                    StdError::generic_err(format!(
                        "invalid value: expected 16 bytes, found {}: {raw}",
                        raw.len()
                    ))
                })
        }
    }

    #[test]
    fn read_write() {
        let mut storage = MockStorage::new();

        storage.write::<TestStore>(&1, &(1, 1));

        let value = storage.read::<TestStore>(&1).unwrap();

        assert_eq!(value, (1, 1));
    }

    #[test]
    fn iterator() {
        let mut storage = MockStorage::new();

        let kvs = [(1, (1, 1)), (2, (1, 2)), (3, (1, 3))];

        for (ref k, ref v) in &kvs {
            storage.write::<TestStore>(k, v);
        }

        let iter_kvs = storage
            .iter::<TestStore>(cosmwasm_std::Order::Ascending)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(iter_kvs, kvs);
    }

    #[test]
    #[allow(non_local_definitions)]
    fn no_overlap() {
        enum A {}
        impl Store for A {
            const PREFIX: Prefix = Prefix::new(&[1]);
            type Key = Bytes;
            type Value = Bytes;
        }

        enum B {}
        impl Store for B {
            const PREFIX: Prefix = Prefix::new(&[1, 1]);
            type Key = Bytes;
            type Value = Bytes;
        }

        trait BytesStore {}
        impl BytesStore for A {}
        impl BytesStore for B {}

        impl<T: BytesStore> KeyCodec<Bytes> for T {
            fn encode_key(key: &Bytes) -> Bytes {
                key.clone()
            }

            fn decode_key(raw: &Bytes) -> StdResult<Bytes> {
                Ok(raw.clone())
            }
        }

        impl<T: BytesStore> ValueCodec<Bytes> for T {
            fn encode_value(value: &Bytes) -> Bytes {
                value.clone()
            }

            fn decode_value(raw: &Bytes) -> StdResult<Bytes> {
                Ok(raw.clone())
            }
        }

        let mut storage = MockStorage::new();

        let a_value = Bytes::new(b"a");
        let b_value = Bytes::new(b"b");

        // this will be stored under 01 00 01
        storage.write::<A>(&Bytes::new(&[1]), &a_value);
        // this will be stored under 01 01 00
        storage.write::<B>(&Bytes::new(&[]), &b_value);

        let value = storage.read::<A>(&Bytes::new(&[1])).unwrap();
        assert_eq!(value, a_value);

        let value = storage.read::<B>(&Bytes::new(&[])).unwrap();
        assert_eq!(value, b_value);
    }

    #[test]
    fn prefix_new() {
        let _ = Prefix::new(b"");
        let _ = Prefix::new(b"a");
        let _ = Prefix::new(b"aa");
        let _ = Prefix::new(b"aaa");
        let _ = Prefix::new(b"aaaa");
        let _ = Prefix::new(b"aaaaa");
    }

    macro_rules! prefix_panic_tests {
        ($($test:ident = $expr:expr;)*) => {
            $(
                #[test]
                #[should_panic = "prefix cannot contain the prefix separator byte 0x00"]
                fn $test() {
                    let _ = Prefix::new($expr);
                }
            )*
        };
    }

    prefix_panic_tests! {
        prefix_new_0 = b"\0";

        prefix_new_a0 = b"a\0";
        prefix_new_aa0 = b"aa\0";

        prefix_new_0a = b"\0a";
        prefix_new_0aa = b"\0aa";

        prefix_new_a0a = b"a\0a";
        prefix_new_aa0aa = b"aa\0aa";

        prefix_new_a0aa = b"a\0aa";
    }
}
