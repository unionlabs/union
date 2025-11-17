use alloc::{format, vec, vec::Vec};

use cosmwasm_std::{StdError, StdResult};
use unionlabs_encoding::{Decode, DecodeAs, Encode, EncodeAs, Encoding};
use unionlabs_primitives::Bytes;

use crate::Store;

/// The value encoding and decoding for a [`Store`].
pub trait ValueCodec<Value> {
    /// Encode the given value for writing to storage.
    ///
    /// # Implementation Note
    ///
    /// This function is expected to be isomorphic with [`ValueCodec::decode_value`].
    fn encode_value(value: &Value) -> Bytes;

    /// Decode the value for this store.
    ///
    /// # Errors
    ///
    /// This function is expected to error iff the key cannot be decoded.
    ///
    /// # Implementation Note
    ///
    /// This function is expected to be isomorphic with [`ValueCodec::encode_value`].
    fn decode_value(raw: &Bytes) -> StdResult<Value>;
}

/// Encode the stored value via the specified [`Encoding`].
///
/// ```rust
/// # use depolama::{Prefix, Store, value::ValueCodecViaEncoding};
/// # use unionlabs_encoding::{Decode, Encode, Encoding};
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
///     type Key = ();
///     type Value = Struct;
/// }
///
/// impl ValueCodecViaEncoding for EthAbiStore {
///     type Encoding = EthAbi;
/// }
/// ```
pub trait ValueCodecViaEncoding: Store<Value: Decode<Self::Encoding>>
where
    for<'a> &'a Self::Value: Encode<Self::Encoding>,
{
    /// The encoding to use.
    type Encoding: Encoding;
}

impl<S> ValueCodec<S::Value> for S
where
    S: ValueCodecViaEncoding<Value: Decode<S::Encoding>>,
    for<'a> &'a S::Value: Encode<S::Encoding>,
{
    fn encode_value(value: &S::Value) -> Bytes {
        value.encode_as::<S::Encoding>().into()
    }

    fn decode_value(raw: &Bytes) -> StdResult<S::Value> {
        <S::Value>::decode_as::<S::Encoding>(raw)
            .map_err(|e| StdError::generic_err(format!("unable to decode: {e:?}")))
    }
}

/// Default unit key encoding to avoid any dependency to an encoding. We encode
/// a single byte because `CosmWasm` has limitations and make it impossible to
/// differentiate non existing key if the value is empty.
pub enum ValueUnitEncoding {}

impl Encoding for ValueUnitEncoding {}
impl Encode<ValueUnitEncoding> for &() {
    fn encode(self) -> Vec<u8> {
        vec![0]
    }
}
impl Decode<ValueUnitEncoding> for () {
    type Error = ();
    fn decode(x: &[u8]) -> Result<Self, Self::Error> {
        match x {
            &[0] => Ok(()),
            _ => Err(()),
        }
    }
}
