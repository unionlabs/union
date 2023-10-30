//! The internal `Value` serialization API.
//!
//! This implementation isn't intended to be public. It may need to change
//! for optimizations or to support new external serialization frameworks.

use crate::{
    fill::{Fill, Slot},
    Error, ValueBag,
};

pub(super) mod cast;
#[cfg(feature = "error")]
pub(super) mod error;
pub(super) mod fill;
pub(super) mod fmt;
#[cfg(feature = "serde1")]
pub(super) mod serde;
#[cfg(feature = "sval1")]
pub(super) mod sval;

// NOTE: It takes less space to have separate variants for the presence
// of a `TypeId` instead of using `Option<T>`, because `TypeId` doesn't
// have a niche value
/// A container for a structured value for a specific kind of visitor.
#[derive(Clone, Copy)]
pub(super) enum Internal<'v> {
    /// A signed integer.
    Signed(i64),
    /// An unsigned integer.
    Unsigned(u64),
    /// An extra large signed integer.
    BigSigned(&'v i128),
    /// An extra large unsigned integer.
    BigUnsigned(&'v u128),
    /// A floating point number.
    Float(f64),
    /// A boolean value.
    Bool(bool),
    /// A UTF8 codepoint.
    Char(char),
    /// A UTF8 string.
    Str(&'v str),
    /// An empty value.
    None,

    /// A value that can be filled.
    Fill(&'v dyn Fill),

    /// A debuggable value.
    AnonDebug(&'v dyn fmt::Debug),
    /// A debuggable value.
    Debug(&'v dyn fmt::DowncastDebug),

    /// A displayable value.
    AnonDisplay(&'v dyn fmt::Display),
    /// A displayable value.
    Display(&'v dyn fmt::DowncastDisplay),

    #[cfg(feature = "error")]
    /// An error.
    AnonError(&'v (dyn error::Error + 'static)),
    #[cfg(feature = "error")]
    /// An error.
    Error(&'v dyn error::DowncastError),

    #[cfg(feature = "sval1")]
    /// A structured value from `sval`.
    AnonSval1(&'v dyn sval::v1::Value),
    #[cfg(feature = "sval1")]
    /// A structured value from `sval`.
    Sval1(&'v dyn sval::v1::DowncastValue),

    #[cfg(feature = "serde1")]
    /// A structured value from `serde`.
    AnonSerde1(&'v dyn serde::v1::Serialize),
    #[cfg(feature = "serde1")]
    /// A structured value from `serde`.
    Serde1(&'v dyn serde::v1::DowncastSerialize),
}

impl<'v> ValueBag<'v> {
    /// Get a value from an internal primitive.
    pub(super) fn from_internal<T>(value: T) -> Self
    where
        T: Into<Internal<'v>>,
    {
        ValueBag {
            inner: value.into(),
        }
    }

    /// Visit the value using an internal visitor.
    #[inline]
    pub(super) fn internal_visit(
        &self,
        visitor: &mut dyn InternalVisitor<'v>,
    ) -> Result<(), Error> {
        self.inner.internal_visit(visitor)
    }
}

impl<'v> Internal<'v> {
    #[inline]
    pub(super) fn internal_visit(self, visitor: &mut dyn InternalVisitor<'v>) -> Result<(), Error> {
        match self {
            Internal::Signed(value) => visitor.i64(value),
            Internal::Unsigned(value) => visitor.u64(value),
            Internal::BigSigned(value) => visitor.i128(value),
            Internal::BigUnsigned(value) => visitor.u128(value),
            Internal::Float(value) => visitor.f64(value),
            Internal::Bool(value) => visitor.bool(value),
            Internal::Char(value) => visitor.char(value),
            Internal::Str(value) => visitor.borrowed_str(value),
            Internal::None => visitor.none(),

            Internal::Fill(value) => value.fill(Slot::new(visitor)),

            Internal::AnonDebug(value) => visitor.debug(value),
            Internal::Debug(value) => visitor.debug(value.as_super()),

            Internal::AnonDisplay(value) => visitor.display(value),
            Internal::Display(value) => visitor.display(value.as_super()),

            #[cfg(feature = "error")]
            Internal::AnonError(value) => visitor.borrowed_error(value),
            #[cfg(feature = "error")]
            Internal::Error(value) => visitor.borrowed_error(value.as_super()),

            #[cfg(feature = "sval1")]
            Internal::AnonSval1(value) => visitor.sval1(value),
            #[cfg(feature = "sval1")]
            Internal::Sval1(value) => visitor.sval1(value.as_super()),

            #[cfg(feature = "serde1")]
            Internal::AnonSerde1(value) => visitor.serde1(value),
            #[cfg(feature = "serde1")]
            Internal::Serde1(value) => visitor.serde1(value.as_super()),
        }
    }
}

/// The internal serialization contract.
pub(super) trait InternalVisitor<'v> {
    fn debug(&mut self, v: &dyn fmt::Debug) -> Result<(), Error>;
    fn display(&mut self, v: &dyn fmt::Display) -> Result<(), Error>;

    fn u64(&mut self, v: u64) -> Result<(), Error>;
    fn i64(&mut self, v: i64) -> Result<(), Error>;
    fn u128(&mut self, v: &u128) -> Result<(), Error>;
    fn borrowed_u128(&mut self, v: &'v u128) -> Result<(), Error> {
        self.u128(v)
    }
    fn i128(&mut self, v: &i128) -> Result<(), Error>;
    fn borrowed_i128(&mut self, v: &'v i128) -> Result<(), Error> {
        self.i128(v)
    }
    fn f64(&mut self, v: f64) -> Result<(), Error>;
    fn bool(&mut self, v: bool) -> Result<(), Error>;
    fn char(&mut self, v: char) -> Result<(), Error>;

    fn str(&mut self, v: &str) -> Result<(), Error>;
    fn borrowed_str(&mut self, v: &'v str) -> Result<(), Error> {
        self.str(v)
    }

    fn none(&mut self) -> Result<(), Error>;

    #[cfg(feature = "error")]
    fn error(&mut self, v: &(dyn error::Error + 'static)) -> Result<(), Error>;
    #[cfg(feature = "error")]
    fn borrowed_error(&mut self, v: &'v (dyn error::Error + 'static)) -> Result<(), Error> {
        self.error(v)
    }

    #[cfg(feature = "sval1")]
    fn sval1(&mut self, v: &dyn sval::v1::Value) -> Result<(), Error>;

    #[cfg(feature = "serde1")]
    fn serde1(&mut self, v: &dyn serde::v1::Serialize) -> Result<(), Error>;
}

impl<'v> From<()> for Internal<'v> {
    #[inline]
    fn from(_: ()) -> Self {
        Internal::None
    }
}

impl<'v> From<u8> for Internal<'v> {
    #[inline]
    fn from(v: u8) -> Self {
        Internal::Unsigned(v as u64)
    }
}

impl<'v> From<u16> for Internal<'v> {
    #[inline]
    fn from(v: u16) -> Self {
        Internal::Unsigned(v as u64)
    }
}

impl<'v> From<u32> for Internal<'v> {
    #[inline]
    fn from(v: u32) -> Self {
        Internal::Unsigned(v as u64)
    }
}

impl<'v> From<u64> for Internal<'v> {
    #[inline]
    fn from(v: u64) -> Self {
        Internal::Unsigned(v)
    }
}

impl<'v> From<usize> for Internal<'v> {
    #[inline]
    fn from(v: usize) -> Self {
        Internal::Unsigned(v as u64)
    }
}

impl<'v> From<i8> for Internal<'v> {
    #[inline]
    fn from(v: i8) -> Self {
        Internal::Signed(v as i64)
    }
}

impl<'v> From<i16> for Internal<'v> {
    #[inline]
    fn from(v: i16) -> Self {
        Internal::Signed(v as i64)
    }
}

impl<'v> From<i32> for Internal<'v> {
    #[inline]
    fn from(v: i32) -> Self {
        Internal::Signed(v as i64)
    }
}

impl<'v> From<i64> for Internal<'v> {
    #[inline]
    fn from(v: i64) -> Self {
        Internal::Signed(v)
    }
}

impl<'v> From<isize> for Internal<'v> {
    #[inline]
    fn from(v: isize) -> Self {
        Internal::Signed(v as i64)
    }
}

impl<'v> From<f32> for Internal<'v> {
    #[inline]
    fn from(v: f32) -> Self {
        Internal::Float(v as f64)
    }
}

impl<'v> From<f64> for Internal<'v> {
    #[inline]
    fn from(v: f64) -> Self {
        Internal::Float(v)
    }
}

impl<'v> From<bool> for Internal<'v> {
    #[inline]
    fn from(v: bool) -> Self {
        Internal::Bool(v)
    }
}

impl<'v> From<char> for Internal<'v> {
    #[inline]
    fn from(v: char) -> Self {
        Internal::Char(v)
    }
}

impl<'v> From<&'v str> for Internal<'v> {
    #[inline]
    fn from(v: &'v str) -> Self {
        Internal::Str(v)
    }
}

impl<'v> From<&'v ()> for Internal<'v> {
    #[inline]
    fn from(_: &'v ()) -> Self {
        Internal::None
    }
}

impl<'v> From<&'v u8> for Internal<'v> {
    #[inline]
    fn from(v: &'v u8) -> Self {
        Internal::Unsigned(*v as u64)
    }
}

impl<'v> From<&'v u16> for Internal<'v> {
    #[inline]
    fn from(v: &'v u16) -> Self {
        Internal::Unsigned(*v as u64)
    }
}

impl<'v> From<&'v u32> for Internal<'v> {
    #[inline]
    fn from(v: &'v u32) -> Self {
        Internal::Unsigned(*v as u64)
    }
}

impl<'v> From<&'v u64> for Internal<'v> {
    #[inline]
    fn from(v: &'v u64) -> Self {
        Internal::Unsigned(*v)
    }
}

impl<'v> From<&'v u128> for Internal<'v> {
    #[inline]
    fn from(v: &'v u128) -> Self {
        Internal::BigUnsigned(v)
    }
}

impl<'v> From<&'v usize> for Internal<'v> {
    #[inline]
    fn from(v: &'v usize) -> Self {
        Internal::Unsigned(*v as u64)
    }
}

impl<'v> From<&'v i8> for Internal<'v> {
    #[inline]
    fn from(v: &'v i8) -> Self {
        Internal::Signed(*v as i64)
    }
}

impl<'v> From<&'v i16> for Internal<'v> {
    #[inline]
    fn from(v: &'v i16) -> Self {
        Internal::Signed(*v as i64)
    }
}

impl<'v> From<&'v i32> for Internal<'v> {
    #[inline]
    fn from(v: &'v i32) -> Self {
        Internal::Signed(*v as i64)
    }
}

impl<'v> From<&'v i64> for Internal<'v> {
    #[inline]
    fn from(v: &'v i64) -> Self {
        Internal::Signed(*v)
    }
}

impl<'v> From<&'v i128> for Internal<'v> {
    #[inline]
    fn from(v: &'v i128) -> Self {
        Internal::BigSigned(v)
    }
}

impl<'v> From<&'v isize> for Internal<'v> {
    #[inline]
    fn from(v: &'v isize) -> Self {
        Internal::Signed(*v as i64)
    }
}

impl<'v> From<&'v f32> for Internal<'v> {
    #[inline]
    fn from(v: &'v f32) -> Self {
        Internal::Float(*v as f64)
    }
}

impl<'v> From<&'v f64> for Internal<'v> {
    #[inline]
    fn from(v: &'v f64) -> Self {
        Internal::Float(*v)
    }
}

impl<'v> From<&'v bool> for Internal<'v> {
    #[inline]
    fn from(v: &'v bool) -> Self {
        Internal::Bool(*v)
    }
}

impl<'v> From<&'v char> for Internal<'v> {
    #[inline]
    fn from(v: &'v char) -> Self {
        Internal::Char(*v)
    }
}

impl<'v, 'u> From<&'v &'u str> for Internal<'v>
where
    'u: 'v,
{
    #[inline]
    fn from(v: &'v &'u str) -> Self {
        Internal::Str(*v)
    }
}

#[cfg(feature = "std")]
mod std_support {
    use super::*;

    use std::string::String;

    impl<'v> From<&'v String> for Internal<'v> {
        #[inline]
        fn from(v: &'v String) -> Self {
            Internal::Str(&**v)
        }
    }
}
