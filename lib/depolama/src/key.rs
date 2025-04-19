use cosmwasm_std::{StdError, StdResult};
use unionlabs_primitives::Bytes;

use crate::Store;

/// The key encoding and decoding for a [`Store`].
///
/// This trait is implemented for all stores with `type Key = ()`, enabling "item store"
/// functionality on [`StorageExt`](crate::StorageExt).
pub trait KeyCodec<Key> {
    /// Encode the given key for writing to storage.
    ///
    /// # Implementation Note
    ///
    /// This function is expected to be isomorphic with [`KeyCodec::decode_key`].
    fn encode_key(key: &Key) -> Bytes;

    /// Decode the key for this store.
    ///
    /// # Errors
    ///
    /// This function is expected to error iff the key cannot be decoded.
    ///
    /// # Implementation Note
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

/// The raw store key for the store for the given key.
pub fn raw_key<S: Store>(key: &S::Key) -> Bytes {
    S::PREFIX
        .iter_with_separator()
        .copied()
        .chain(S::encode_key(key))
        .collect()
}
