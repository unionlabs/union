//! Field elements.

use crate::{
    bigint::{ArrayEncoding, ByteArray, Integer},
    Curve,
};
use generic_array::{typenum::Unsigned, GenericArray};

/// Size of serialized field elements of this elliptic curve.
pub type FieldBytesSize<C> = <C as Curve>::FieldBytesSize;

/// Byte representation of a base/scalar field element of a given curve.
pub type FieldBytes<C> = GenericArray<u8, FieldBytesSize<C>>;

/// Trait for decoding/encoding `Curve::Uint` from/to [`FieldBytes`] using
/// curve-specific rules.
///
/// Namely a curve's modulus may be smaller than the big integer type used to
/// internally represent field elements (since the latter are multiples of the
/// limb size), such as in the case of curves like NIST P-224 and P-521, and so
/// it may need to be padded/truncated to the right length.
///
/// Additionally, different curves have different endianness conventions, also
/// captured here.
pub trait FieldBytesEncoding<C>: ArrayEncoding + Integer
where
    C: Curve,
{
    /// Decode unsigned integer from serialized field element.
    ///
    /// The default implementation assumes a big endian encoding.
    fn decode_field_bytes(field_bytes: &FieldBytes<C>) -> Self {
        debug_assert!(field_bytes.len() <= Self::ByteSize::USIZE);
        let mut byte_array = ByteArray::<Self>::default();
        let offset = Self::ByteSize::USIZE.saturating_sub(field_bytes.len());
        byte_array[offset..].copy_from_slice(field_bytes);
        Self::from_be_byte_array(byte_array)
    }

    /// Encode unsigned integer into serialized field element.
    ///
    /// The default implementation assumes a big endian encoding.
    fn encode_field_bytes(&self) -> FieldBytes<C> {
        let mut field_bytes = FieldBytes::<C>::default();
        debug_assert!(field_bytes.len() <= Self::ByteSize::USIZE);

        let offset = Self::ByteSize::USIZE.saturating_sub(field_bytes.len());
        field_bytes.copy_from_slice(&self.to_be_byte_array()[offset..]);
        field_bytes
    }
}
