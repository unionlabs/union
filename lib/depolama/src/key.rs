use alloc::{format, vec, vec::Vec};

use cosmwasm_std::{StdError, StdResult};
use unionlabs_encoding::{Decode, DecodeAs, Encode, EncodeAs, Encoding};
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

/// The raw store key for the store for the given key.
pub fn raw_key<S: Store>(key: &S::Key) -> Bytes {
    S::PREFIX
        .iter_with_separator()
        .copied()
        .chain(S::encode_key(key))
        .collect()
}

/// Encode the stored key via the specified [`Encoding`].
///
/// ```rust
/// # use depolama::{Prefix, Store, key::KeyCodecViaEncoding, value::ValueCodecViaEncoding, key::KeyUnitEncoding};
/// # use unionlabs_encoding::{Decode, Encode ,Encoding};
/// # enum EthAbi {}
/// # impl Encoding for EthAbi {}
/// # impl Encode<EthAbi> for &'_ Struct {
/// #     fn encode(self) -> Vec<u8> { todo!() }
/// # }
/// # impl Decode<EthAbi> for Struct {
/// #     type Error = ();
/// #     fn decode(_: &[u8]) -> Result<Self, Self::Error> { todo!() }
/// # }
/// struct Struct {
///     // some fields
/// }
///
/// enum EthAbiStore {}
///
/// impl Store for EthAbiStore {
///     const PREFIX: Prefix = Prefix::new(b"prefix");
///
///     type Key = Struct;
///     type Value = ();
/// }
///
/// impl ValueCodecViaEncoding for EthAbiStore {
///     type Encoding = KeyUnitEncoding;
/// }
/// impl KeyCodecViaEncoding for EthAbiStore {
///     type Encoding = EthAbi;
/// }
/// ```
pub trait KeyCodecViaEncoding: Store<Key: Decode<Self::Encoding>>
where
    for<'a> &'a Self::Key: Encode<Self::Encoding>,
{
    /// The encoding to use.
    type Encoding: Encoding;
}

impl<S> KeyCodec<S::Key> for S
where
    S: KeyCodecViaEncoding<Key: Decode<S::Encoding>>,
    for<'a> &'a S::Key: Encode<S::Encoding>,
{
    fn encode_key(value: &S::Key) -> Bytes {
        value.encode_as::<S::Encoding>().into()
    }

    fn decode_key(raw: &Bytes) -> StdResult<S::Key> {
        <S::Key>::decode_as::<S::Encoding>(raw)
            .map_err(|e| StdError::generic_err(format!("unable to decode: {e:?}")))
    }
}

/// Default unit key encoding to avoid any dependency to an encoding.
pub struct KeyUnitEncoding;

impl Encoding for KeyUnitEncoding {}
impl Encode<KeyUnitEncoding> for &() {
    fn encode(self) -> Vec<u8> {
        vec![]
    }
}
impl Decode<KeyUnitEncoding> for () {
    type Error = ();
    fn decode(_: &[u8]) -> Result<Self, Self::Error> {
        Ok(())
    }
}

impl<T: Store<Key = ()>> KeyCodecViaEncoding for T {
    type Encoding = KeyUnitEncoding;
}
