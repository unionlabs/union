//! Integration between `Value` and `sval`.
//!
//! This module allows any `Value` to implement the `Value` trait,
//! and for any `Value` to be captured as a `Value`.

use crate::{
    fill::Slot,
    internal::{Internal, InternalVisitor},
    std::{any::Any, fmt},
    Error, ValueBag,
};

impl<'v> ValueBag<'v> {
    /// Get a value from a structured type.
    ///
    /// This method will attempt to capture the given value as a well-known primitive
    /// before resorting to using its `Value` implementation.
    pub fn capture_sval1<T>(value: &'v T) -> Self
    where
        T: Value + 'static,
    {
        Self::try_capture(value).unwrap_or(ValueBag {
            inner: Internal::Sval1(value),
        })
    }

    /// Get a value from a structured type without capturing support.
    pub fn from_sval1<T>(value: &'v T) -> Self
    where
        T: Value,
    {
        ValueBag {
            inner: Internal::AnonSval1(value),
        }
    }

    /// Get a value from an erased structured type.
    #[inline]
    pub fn from_dyn_sval1(value: &'v dyn Value) -> Self {
        ValueBag {
            inner: Internal::AnonSval1(value),
        }
    }
}

pub(crate) trait DowncastValue {
    fn as_any(&self) -> &dyn Any;
    fn as_super(&self) -> &dyn Value;
}

impl<T: Value + 'static> DowncastValue for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_super(&self) -> &dyn Value {
        self
    }
}

impl<'s, 'f> Slot<'s, 'f> {
    /// Fill the slot with a structured value.
    ///
    /// The given value doesn't need to satisfy any particular lifetime constraints.
    pub fn fill_sval1<T>(self, value: T) -> Result<(), Error>
    where
        T: Value,
    {
        self.fill(|visitor| visitor.sval1(&value))
    }

    /// Fill the slot with a structured value.
    pub fn fill_dyn_sval1(self, value: &dyn Value) -> Result<(), Error> {
        self.fill(|visitor| visitor.sval1(value))
    }
}

impl<'v> Value for ValueBag<'v> {
    fn stream(&self, s: &mut sval1_lib::value::Stream) -> sval1_lib::value::Result {
        struct Sval1Visitor<'a, 'b: 'a>(&'a mut sval1_lib::value::Stream<'b>);

        impl<'a, 'b: 'a, 'v> InternalVisitor<'v> for Sval1Visitor<'a, 'b> {
            fn debug(&mut self, v: &dyn fmt::Debug) -> Result<(), Error> {
                self.0.debug(v).map_err(Error::from_sval1)
            }

            fn display(&mut self, v: &dyn fmt::Display) -> Result<(), Error> {
                self.0.display(v).map_err(Error::from_sval1)
            }

            fn u64(&mut self, v: u64) -> Result<(), Error> {
                self.0.u64(v).map_err(Error::from_sval1)
            }

            fn i64(&mut self, v: i64) -> Result<(), Error> {
                self.0.i64(v).map_err(Error::from_sval1)
            }

            fn u128(&mut self, v: &u128) -> Result<(), Error> {
                self.0.u128(*v).map_err(Error::from_sval1)
            }

            fn i128(&mut self, v: &i128) -> Result<(), Error> {
                self.0.i128(*v).map_err(Error::from_sval1)
            }

            fn f64(&mut self, v: f64) -> Result<(), Error> {
                self.0.f64(v).map_err(Error::from_sval1)
            }

            fn bool(&mut self, v: bool) -> Result<(), Error> {
                self.0.bool(v).map_err(Error::from_sval1)
            }

            fn char(&mut self, v: char) -> Result<(), Error> {
                self.0.char(v).map_err(Error::from_sval1)
            }

            fn str(&mut self, v: &str) -> Result<(), Error> {
                self.0.str(v).map_err(Error::from_sval1)
            }

            fn none(&mut self) -> Result<(), Error> {
                self.0.none().map_err(Error::from_sval1)
            }

            #[cfg(feature = "error")]
            fn error(&mut self, v: &(dyn std::error::Error + 'static)) -> Result<(), Error> {
                self.0.error(v).map_err(Error::from_sval1)
            }

            fn sval1(&mut self, v: &dyn Value) -> Result<(), Error> {
                self.0.any(v).map_err(Error::from_sval1)
            }

            #[cfg(feature = "serde1")]
            fn serde1(
                &mut self,
                v: &dyn crate::internal::serde::v1::Serialize,
            ) -> Result<(), Error> {
                crate::internal::serde::v1::sval1(self.0, v)
            }
        }

        self.internal_visit(&mut Sval1Visitor(s))
            .map_err(Error::into_sval1)?;

        Ok(())
    }
}

pub use sval1_lib::value::Value;

pub(in crate::internal) fn fmt(f: &mut fmt::Formatter, v: &dyn Value) -> Result<(), Error> {
    sval1_lib::fmt::debug(f, v)?;
    Ok(())
}

#[cfg(feature = "serde1")]
pub(in crate::internal) fn serde<S>(s: S, v: &dyn Value) -> Result<S::Ok, S::Error>
where
    S: serde1_lib::Serializer,
{
    sval1_lib::serde::v1::serialize(s, v)
}

pub(crate) fn internal_visit<'v>(
    v: &dyn Value,
    visitor: &mut dyn InternalVisitor<'v>,
) -> Result<(), Error> {
    struct VisitorStream<'a, 'v>(&'a mut dyn InternalVisitor<'v>);

    impl<'a, 'v> sval1_lib::stream::Stream for VisitorStream<'a, 'v> {
        fn fmt(&mut self, v: sval1_lib::stream::Arguments) -> sval1_lib::stream::Result {
            self.0.display(&v).map_err(Error::into_sval1)?;
            Ok(())
        }

        #[cfg(feature = "error")]
        fn error(&mut self, v: sval1_lib::stream::Source) -> sval1_lib::stream::Result {
            self.0.error(v.get()).map_err(Error::into_sval1)?;
            Ok(())
        }

        fn i64(&mut self, v: i64) -> sval1_lib::stream::Result {
            self.0.i64(v).map_err(Error::into_sval1)?;
            Ok(())
        }

        fn u64(&mut self, v: u64) -> sval1_lib::stream::Result {
            self.0.u64(v).map_err(Error::into_sval1)?;
            Ok(())
        }

        fn i128(&mut self, v: i128) -> sval1_lib::stream::Result {
            self.0.i128(&v).map_err(Error::into_sval1)?;
            Ok(())
        }

        fn u128(&mut self, v: u128) -> sval1_lib::stream::Result {
            self.0.u128(&v).map_err(Error::into_sval1)?;
            Ok(())
        }

        fn f64(&mut self, v: f64) -> sval1_lib::stream::Result {
            self.0.f64(v).map_err(Error::into_sval1)?;
            Ok(())
        }

        fn bool(&mut self, v: bool) -> sval1_lib::stream::Result {
            self.0.bool(v).map_err(Error::into_sval1)?;
            Ok(())
        }

        fn char(&mut self, v: char) -> sval1_lib::stream::Result {
            self.0.char(v).map_err(Error::into_sval1)?;
            Ok(())
        }

        fn str(&mut self, s: &str) -> sval1_lib::stream::Result {
            self.0.str(s).map_err(Error::into_sval1)?;
            Ok(())
        }
    }

    let mut visitor = VisitorStream(visitor);
    sval1_lib::stream(&mut visitor, v).map_err(Error::from_sval1)?;

    Ok(())
}

impl Error {
    pub(in crate::internal) fn from_sval1(_: sval1_lib::Error) -> Self {
        Error::msg("`sval` serialization failed")
    }

    pub(in crate::internal) fn into_sval1(self) -> sval1_lib::Error {
        sval1_lib::Error::msg("`sval` serialization failed")
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    use super::*;
    use crate::test::*;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn sval1_capture() {
        assert_eq!(ValueBag::capture_sval1(&42u64).to_token(), Token::U64(42));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn sval1_capture_cast() {
        assert_eq!(
            42u64,
            ValueBag::capture_sval1(&42u64)
                .to_u64()
                .expect("invalid value")
        );

        assert_eq!(
            "a string",
            ValueBag::capture_sval1(&"a string")
                .to_borrowed_str()
                .expect("invalid value")
        );

        #[cfg(feature = "std")]
        assert_eq!(
            "a string",
            ValueBag::capture_sval1(&"a string")
                .to_str()
                .expect("invalid value")
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn sval1_from_cast() {
        assert_eq!(
            42u64,
            ValueBag::from_sval1(&42u64)
                .to_u64()
                .expect("invalid value")
        );

        #[cfg(feature = "std")]
        assert_eq!(
            "a string",
            ValueBag::from_sval1(&"a string")
                .to_str()
                .expect("invalid value")
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn sval1_downcast() {
        #[derive(Debug, PartialEq, Eq)]
        struct Timestamp(usize);

        impl Value for Timestamp {
            fn stream(&self, stream: &mut sval1_lib::value::Stream) -> sval1_lib::value::Result {
                stream.u64(self.0 as u64)
            }
        }

        let ts = Timestamp(42);

        assert_eq!(
            &ts,
            ValueBag::capture_sval1(&ts)
                .downcast_ref::<Timestamp>()
                .expect("invalid value")
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn sval1_structured() {
        let value = ValueBag::from(42u64);
        let expected = vec![sval1_lib::test::Token::Unsigned(42)];

        assert_eq!(sval1_lib::test::tokens(value), expected);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn sval1_debug() {
        struct TestSval;

        impl Value for TestSval {
            fn stream(&self, stream: &mut sval1_lib::value::Stream) -> sval1_lib::value::Result {
                stream.u64(42)
            }
        }

        assert_eq!(
            format!("{:04?}", 42u64),
            format!("{:04?}", ValueBag::capture_sval1(&TestSval)),
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn sval1_visit() {
        ValueBag::from_dyn_sval1(&42u64)
            .visit(TestVisit)
            .expect("failed to visit value");
        ValueBag::from_dyn_sval1(&-42i64)
            .visit(TestVisit)
            .expect("failed to visit value");
        ValueBag::from_dyn_sval1(&11f64)
            .visit(TestVisit)
            .expect("failed to visit value");
        ValueBag::from_dyn_sval1(&true)
            .visit(TestVisit)
            .expect("failed to visit value");
        ValueBag::from_dyn_sval1(&"some string")
            .visit(TestVisit)
            .expect("failed to visit value");
        ValueBag::from_dyn_sval1(&'n')
            .visit(TestVisit)
            .expect("failed to visit value");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg(feature = "serde1")]
    fn sval1_serde1() {
        use serde1_test::{assert_ser_tokens, Token};

        struct TestSval;

        impl Value for TestSval {
            fn stream(&self, stream: &mut sval1_lib::value::Stream) -> sval1_lib::value::Result {
                stream.u64(42)
            }
        }

        assert_ser_tokens(&ValueBag::capture_sval1(&TestSval), &[Token::U64(42)]);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg(feature = "error")]
    fn sval1_visit_error() {
        use crate::{
            internal::sval::v1 as sval,
            std::{error, io},
        };

        let err: &(dyn error::Error + 'static) = &io::Error::from(io::ErrorKind::Other);
        let value: &dyn sval::Value = &err;

        // Ensure that an error captured through `sval` can be visited as an error
        ValueBag::from_dyn_sval1(value)
            .visit(TestVisit)
            .expect("failed to visit value");
    }

    #[cfg(feature = "std")]
    mod std_support {
        use super::*;

        use crate::std::borrow::ToOwned;

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        fn sval1_cast() {
            assert_eq!(
                "a string",
                ValueBag::capture_sval1(&"a string".to_owned())
                    .to_str()
                    .expect("invalid value")
            );
        }
    }
}
