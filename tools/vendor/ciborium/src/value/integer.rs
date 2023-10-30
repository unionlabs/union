// SPDX-License-Identifier: Apache-2.0

macro_rules! implfrom {
    ($( $(#[$($attr:meta)+])? $t:ident)+) => {
        $(
            $(#[$($attr)+])?
            impl From<$t> for Integer {
                #[inline]
                fn from(value: $t) -> Self {
                    Self(value as _)
                }
            }

            impl TryFrom<Integer> for $t {
                type Error = core::num::TryFromIntError;

                #[inline]
                fn try_from(value: Integer) -> Result<Self, Self::Error> {
                    $t::try_from(value.0)
                }
            }
        )+
    };
}

/// An abstract integer value
///
/// This opaque type represents an integer value which can be encoded in CBOR
/// without resulting to big integer encoding. Larger values may be encoded
/// using the big integer encoding as described in the CBOR RFC. See the
/// implementations for 128-bit integer conversions on `Value` for more
/// details.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Integer(i128);

implfrom! {
    u8 u16 u32 u64
    i8 i16 i32 i64

    #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
    usize

    #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
    isize
}

impl TryFrom<i128> for Integer {
    type Error = core::num::TryFromIntError;

    #[inline]
    fn try_from(value: i128) -> Result<Self, Self::Error> {
        u64::try_from(match value.is_negative() {
            false => value,
            true => value ^ !0,
        })?;

        Ok(Integer(value))
    }
}

impl TryFrom<u128> for Integer {
    type Error = core::num::TryFromIntError;

    #[inline]
    fn try_from(value: u128) -> Result<Self, Self::Error> {
        Ok(Self(u64::try_from(value)?.into()))
    }
}

impl From<Integer> for i128 {
    #[inline]
    fn from(value: Integer) -> Self {
        value.0
    }
}

impl TryFrom<Integer> for u128 {
    type Error = core::num::TryFromIntError;

    #[inline]
    fn try_from(value: Integer) -> Result<Self, Self::Error> {
        u128::try_from(value.0)
    }
}
