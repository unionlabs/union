//! Coerce a `Value` into some concrete types.
//!
//! These operations are cheap when the captured value is a simple primitive,
//! but may end up executing arbitrary caller code if the value is complex.
//! They will also attempt to downcast erased types into a primitive where possible.

use crate::std::{
    convert::{TryFrom, TryInto},
    fmt,
};

#[cfg(feature = "std")]
use crate::std::{borrow::ToOwned, string::String};

use super::{Internal, InternalVisitor};
use crate::{Error, ValueBag};

mod primitive;

impl<'v> ValueBag<'v> {
    /// Try capture a raw value.
    ///
    /// This method will return `Some` if the value is a simple primitive
    /// that can be captured without losing its structure. In other cases
    /// this method will return `None`.
    pub fn try_capture<T>(value: &'v T) -> Option<Self>
    where
        T: ?Sized + 'static,
    {
        primitive::from_any(value).map(|inner| ValueBag { inner })
    }

    /// Try get a `u64` from this value.
    ///
    /// This method is cheap for primitive types, but may call arbitrary
    /// serialization implementations for complex ones.
    pub fn to_u64(&self) -> Option<u64> {
        self.inner.cast().into_u64()
    }

    /// Try get a `i64` from this value.
    ///
    /// This method is cheap for primitive types, but may call arbitrary
    /// serialization implementations for complex ones.
    pub fn to_i64(&self) -> Option<i64> {
        self.inner.cast().into_i64()
    }

    /// Try get a `u128` from this value.
    ///
    /// This method is cheap for primitive types, but may call arbitrary
    /// serialization implementations for complex ones.
    pub fn to_u128(&self) -> Option<u128> {
        self.inner.cast().into_u128()
    }

    /// Try get a `i128` from this value.
    ///
    /// This method is cheap for primitive types, but may call arbitrary
    /// serialization implementations for complex ones.
    pub fn to_i128(&self) -> Option<i128> {
        self.inner.cast().into_i128()
    }

    /// Try get a `f64` from this value.
    ///
    /// This method is cheap for primitive types, but may call arbitrary
    /// serialization implementations for complex ones.
    pub fn to_f64(&self) -> Option<f64> {
        self.inner.cast().into_f64()
    }

    /// Try get a `bool` from this value.
    ///
    /// This method is cheap for primitive types, but may call arbitrary
    /// serialization implementations for complex ones.
    pub fn to_bool(&self) -> Option<bool> {
        self.inner.cast().into_bool()
    }

    /// Try get a `char` from this value.
    ///
    /// This method is cheap for primitive types, but may call arbitrary
    /// serialization implementations for complex ones.
    pub fn to_char(&self) -> Option<char> {
        self.inner.cast().into_char()
    }

    /// Try get a `str` from this value.
    ///
    /// This method is cheap for primitive types. It won't allocate an owned
    /// `String` if the value is a complex type.
    pub fn to_borrowed_str(&self) -> Option<&str> {
        self.inner.cast().into_borrowed_str()
    }

    /// Check whether this value can be downcast to `T`.
    pub fn is<T: 'static>(&self) -> bool {
        self.downcast_ref::<T>().is_some()
    }

    /// Try downcast this value to `T`.
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        match self.inner {
            Internal::Debug(value) => value.as_any().downcast_ref(),
            Internal::Display(value) => value.as_any().downcast_ref(),
            #[cfg(feature = "error")]
            Internal::Error(value) => value.as_any().downcast_ref(),
            #[cfg(feature = "sval1")]
            Internal::Sval1(value) => value.as_any().downcast_ref(),
            #[cfg(feature = "serde1")]
            Internal::Serde1(value) => value.as_any().downcast_ref(),
            _ => None,
        }
    }
}

impl<'v> Internal<'v> {
    /// Cast the inner value to another type.
    #[inline]
    fn cast(self) -> Cast<'v> {
        struct CastVisitor<'v>(Cast<'v>);

        impl<'v> InternalVisitor<'v> for CastVisitor<'v> {
            #[inline]
            fn debug(&mut self, _: &dyn fmt::Debug) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn display(&mut self, _: &dyn fmt::Display) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn u64(&mut self, v: u64) -> Result<(), Error> {
                self.0 = Cast::Unsigned(v);
                Ok(())
            }

            #[inline]
            fn i64(&mut self, v: i64) -> Result<(), Error> {
                self.0 = Cast::Signed(v);
                Ok(())
            }

            #[inline]
            fn u128(&mut self, v: &u128) -> Result<(), Error> {
                self.0 = Cast::BigUnsigned(*v);
                Ok(())
            }

            #[inline]
            fn i128(&mut self, v: &i128) -> Result<(), Error> {
                self.0 = Cast::BigSigned(*v);
                Ok(())
            }

            #[inline]
            fn f64(&mut self, v: f64) -> Result<(), Error> {
                self.0 = Cast::Float(v);
                Ok(())
            }

            #[inline]
            fn bool(&mut self, v: bool) -> Result<(), Error> {
                self.0 = Cast::Bool(v);
                Ok(())
            }

            #[inline]
            fn char(&mut self, v: char) -> Result<(), Error> {
                self.0 = Cast::Char(v);
                Ok(())
            }

            #[cfg(feature = "std")]
            #[inline]
            fn str(&mut self, s: &str) -> Result<(), Error> {
                self.0 = Cast::String(s.to_owned());
                Ok(())
            }

            #[cfg(not(feature = "std"))]
            #[inline]
            fn str(&mut self, _: &str) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn borrowed_str(&mut self, v: &'v str) -> Result<(), Error> {
                self.0 = Cast::Str(v);
                Ok(())
            }

            #[inline]
            fn none(&mut self) -> Result<(), Error> {
                self.0 = Cast::None;
                Ok(())
            }

            #[cfg(feature = "error")]
            #[inline]
            fn error(&mut self, _: &dyn super::error::Error) -> Result<(), Error> {
                Ok(())
            }

            #[cfg(feature = "sval1")]
            #[inline]
            fn sval1(&mut self, v: &dyn super::sval::v1::Value) -> Result<(), Error> {
                super::sval::v1::internal_visit(v, self)
            }

            #[cfg(feature = "serde1")]
            #[inline]
            fn serde1(&mut self, v: &dyn super::serde::v1::Serialize) -> Result<(), Error> {
                super::serde::v1::internal_visit(v, self)
            }
        }

        match &self {
            Internal::Signed(value) => Cast::Signed(*value),
            Internal::Unsigned(value) => Cast::Unsigned(*value),
            Internal::BigSigned(value) => Cast::BigSigned(**value),
            Internal::BigUnsigned(value) => Cast::BigUnsigned(**value),
            Internal::Float(value) => Cast::Float(*value),
            Internal::Bool(value) => Cast::Bool(*value),
            Internal::Char(value) => Cast::Char(*value),
            Internal::Str(value) => Cast::Str(*value),
            Internal::None => Cast::None,
            other => {
                // If the erased value isn't a primitive then we visit it
                let mut cast = CastVisitor(Cast::None);
                let _ = other.internal_visit(&mut cast);
                cast.0
            }
        }
    }
}

pub(in crate::internal) enum Cast<'v> {
    Signed(i64),
    Unsigned(u64),
    BigSigned(i128),
    BigUnsigned(u128),
    Float(f64),
    Bool(bool),
    Char(char),
    Str(&'v str),
    None,
    #[cfg(feature = "std")]
    String(String),
}

impl<'v> Cast<'v> {
    #[inline]
    fn into_borrowed_str(self) -> Option<&'v str> {
        if let Cast::Str(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    fn into_u64(self) -> Option<u64> {
        match self {
            Cast::Unsigned(value) => Some(value),
            Cast::BigUnsigned(value) => value.try_into().ok(),
            Cast::Signed(value) => value.try_into().ok(),
            Cast::BigSigned(value) => value.try_into().ok(),
            _ => None,
        }
    }

    #[inline]
    fn into_i64(self) -> Option<i64> {
        match self {
            Cast::Signed(value) => Some(value),
            Cast::BigSigned(value) => value.try_into().ok(),
            Cast::Unsigned(value) => value.try_into().ok(),
            Cast::BigUnsigned(value) => value.try_into().ok(),
            _ => None,
        }
    }

    #[inline]
    fn into_u128(self) -> Option<u128> {
        match self {
            Cast::BigUnsigned(value) => Some(value),
            Cast::Unsigned(value) => Some(value.into()),
            Cast::Signed(value) => value.try_into().ok(),
            Cast::BigSigned(value) => value.try_into().ok(),
            _ => None,
        }
    }

    #[inline]
    fn into_i128(self) -> Option<i128> {
        match self {
            Cast::BigSigned(value) => Some(value),
            Cast::Signed(value) => Some(value.into()),
            Cast::Unsigned(value) => value.try_into().ok(),
            Cast::BigUnsigned(value) => value.try_into().ok(),
            _ => None,
        }
    }

    #[inline]
    fn into_f64(self) -> Option<f64> {
        match self {
            Cast::Float(value) => Some(value),
            Cast::Unsigned(value) => u32::try_from(value)
                .ok()
                .and_then(|value| value.try_into().ok()),
            Cast::Signed(value) => i32::try_from(value)
                .ok()
                .and_then(|value| value.try_into().ok()),
            Cast::BigUnsigned(value) => u32::try_from(value)
                .ok()
                .and_then(|value| value.try_into().ok()),
            Cast::BigSigned(value) => i32::try_from(value)
                .ok()
                .and_then(|value| value.try_into().ok()),
            _ => None,
        }
    }

    #[inline]
    fn into_char(self) -> Option<char> {
        if let Cast::Char(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    fn into_bool(self) -> Option<bool> {
        if let Cast::Bool(value) = self {
            Some(value)
        } else {
            None
        }
    }
}

#[cfg(feature = "std")]
mod std_support {
    use super::*;

    use crate::std::borrow::Cow;

    impl<'v> ValueBag<'v> {
        /// Try get a `str` from this value.
        ///
        /// This method is cheap for primitive types, but may call arbitrary
        /// serialization implementations for complex ones. If the serialization
        /// implementation produces a short lived string it will be allocated.
        #[inline]
        pub fn to_str(&self) -> Option<Cow<str>> {
            self.inner.cast().into_str()
        }
    }

    impl<'v> Cast<'v> {
        #[inline]
        pub(super) fn into_str(self) -> Option<Cow<'v, str>> {
            match self {
                Cast::Str(value) => Some(value.into()),
                Cast::String(value) => Some(value.into()),
                _ => None,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        #[cfg(target_arch = "wasm32")]
        use wasm_bindgen_test::*;

        use crate::{std::borrow::ToOwned, test::IntoValueBag, ValueBag};

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        fn primitive_cast() {
            let short_lived = "a string".to_owned();
            assert_eq!(
                "a string",
                (&*short_lived)
                    .into_value_bag()
                    .to_borrowed_str()
                    .expect("invalid value")
            );
            assert_eq!(
                "a string",
                &*"a string".into_value_bag().to_str().expect("invalid value")
            );
            assert_eq!(
                "a string",
                (&*short_lived)
                    .into_value_bag()
                    .to_borrowed_str()
                    .expect("invalid value")
            );
            assert_eq!(
                "a string",
                ValueBag::try_capture(&short_lived)
                    .expect("invalid value")
                    .to_borrowed_str()
                    .expect("invalid value")
            );
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    use super::*;

    use crate::std::string::ToString;

    use crate::test::IntoValueBag;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn primitive_capture_str() {
        let s: &str = &"short lived".to_string();
        assert_eq!(
            "short lived",
            ValueBag::try_capture(s)
                .unwrap()
                .to_borrowed_str()
                .expect("invalid value")
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn primitive_cast() {
        assert_eq!(
            "a string",
            "a string"
                .into_value_bag()
                .to_borrowed_str()
                .expect("invalid value")
        );

        assert_eq!(1u64, 1u8.into_value_bag().to_u64().expect("invalid value"));
        assert_eq!(1u64, 1u16.into_value_bag().to_u64().expect("invalid value"));
        assert_eq!(1u64, 1u32.into_value_bag().to_u64().expect("invalid value"));
        assert_eq!(1u64, 1u64.into_value_bag().to_u64().expect("invalid value"));
        assert_eq!(
            1u64,
            1usize.into_value_bag().to_u64().expect("invalid value")
        );
        assert_eq!(
            1u128,
            1u128.into_value_bag().to_u128().expect("invalid value")
        );

        assert_eq!(
            -1i64,
            -1i8.into_value_bag().to_i64().expect("invalid value")
        );
        assert_eq!(
            -1i64,
            -1i8.into_value_bag().to_i64().expect("invalid value")
        );
        assert_eq!(
            -1i64,
            -1i8.into_value_bag().to_i64().expect("invalid value")
        );
        assert_eq!(
            -1i64,
            -1i64.into_value_bag().to_i64().expect("invalid value")
        );
        assert_eq!(
            -1i64,
            -1isize.into_value_bag().to_i64().expect("invalid value")
        );
        assert_eq!(
            -1i128,
            -1i128.into_value_bag().to_i128().expect("invalid value")
        );

        assert!(1f64.into_value_bag().to_f64().is_some());
        assert!(1u64.into_value_bag().to_f64().is_some());
        assert!((-1i64).into_value_bag().to_f64().is_some());
        assert!(1u128.into_value_bag().to_f64().is_some());
        assert!((-1i128).into_value_bag().to_f64().is_some());

        assert!(u64::MAX.into_value_bag().to_u128().is_some());
        assert!(i64::MIN.into_value_bag().to_i128().is_some());
        assert!(i64::MAX.into_value_bag().to_u64().is_some());

        assert!((-1i64).into_value_bag().to_u64().is_none());
        assert!(u64::MAX.into_value_bag().to_i64().is_none());
        assert!(u64::MAX.into_value_bag().to_f64().is_none());

        assert!(i128::MAX.into_value_bag().to_i64().is_none());
        assert!(u128::MAX.into_value_bag().to_u64().is_none());

        assert!(1f64.into_value_bag().to_u64().is_none());

        assert_eq!('a', 'a'.into_value_bag().to_char().expect("invalid value"));
        assert_eq!(
            true,
            true.into_value_bag().to_bool().expect("invalid value")
        );
    }
}
