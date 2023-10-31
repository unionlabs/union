//! Integration between `Value` and `serde`.
//!
//! This module allows any `Value` to implement the `Serialize` trait,
//! and for any `Serialize` to be captured as a `Value`.

use crate::{
    fill::Slot,
    internal::{Internal, InternalVisitor},
    std::{any::Any, fmt},
    Error, ValueBag,
};

use serde1_lib::ser::{Error as SerdeError, Impossible};

impl<'v> ValueBag<'v> {
    /// Get a value from a structured type.
    ///
    /// This method will attempt to capture the given value as a well-known primitive
    /// before resorting to using its `Value` implementation.
    pub fn capture_serde1<T>(value: &'v T) -> Self
    where
        T: serde1_lib::Serialize + 'static,
    {
        Self::try_capture(value).unwrap_or(ValueBag {
            inner: Internal::Serde1(value),
        })
    }

    /// Get a value from a structured type without capturing support.
    pub fn from_serde1<T>(value: &'v T) -> Self
    where
        T: serde1_lib::Serialize,
    {
        ValueBag {
            inner: Internal::AnonSerde1(value),
        }
    }
}

pub(crate) trait DowncastSerialize {
    fn as_any(&self) -> &dyn Any;
    fn as_super(&self) -> &dyn Serialize;
}

impl<T: serde1_lib::Serialize + 'static> DowncastSerialize for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_super(&self) -> &dyn Serialize {
        self
    }
}

impl<'s, 'f> Slot<'s, 'f> {
    /// Fill the slot with a structured value.
    ///
    /// The given value doesn't need to satisfy any particular lifetime constraints.
    pub fn fill_serde1<T>(self, value: T) -> Result<(), Error>
    where
        T: serde1_lib::Serialize,
    {
        self.fill(|visitor| visitor.serde1(&value))
    }
}

impl<'v> serde1_lib::Serialize for ValueBag<'v> {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde1_lib::Serializer,
    {
        struct Serde1Visitor<S>
        where
            S: serde1_lib::Serializer,
        {
            inner: Option<S>,
            result: Option<Result<S::Ok, S::Error>>,
        }

        impl<S> Serde1Visitor<S>
        where
            S: serde1_lib::Serializer,
        {
            fn result(&self) -> Result<(), Error> {
                match self.result {
                    Some(Ok(_)) => Ok(()),
                    Some(Err(_)) | None => Err(Error::serde()),
                }
            }

            fn serializer(&mut self) -> Result<S, Error> {
                self.inner.take().ok_or_else(|| Error::serde())
            }

            fn into_result(self) -> Result<S::Ok, S::Error> {
                self.result
                    .unwrap_or_else(|| Err(S::Error::custom("`serde` serialization failed")))
            }
        }

        impl<'v, S> InternalVisitor<'v> for Serde1Visitor<S>
        where
            S: serde1_lib::Serializer,
        {
            fn debug(&mut self, v: &dyn fmt::Debug) -> Result<(), Error> {
                struct DebugToDisplay<T>(T);

                impl<T> fmt::Display for DebugToDisplay<T>
                where
                    T: fmt::Debug,
                {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        fmt::Debug::fmt(&self.0, f)
                    }
                }

                self.result = Some(self.serializer()?.collect_str(&DebugToDisplay(v)));
                self.result()
            }

            fn display(&mut self, v: &dyn fmt::Display) -> Result<(), Error> {
                self.result = Some(self.serializer()?.collect_str(v));
                self.result()
            }

            fn u64(&mut self, v: u64) -> Result<(), Error> {
                self.result = Some(self.serializer()?.serialize_u64(v));
                self.result()
            }

            fn i64(&mut self, v: i64) -> Result<(), Error> {
                self.result = Some(self.serializer()?.serialize_i64(v));
                self.result()
            }

            fn u128(&mut self, v: &u128) -> Result<(), Error> {
                self.result = Some(self.serializer()?.serialize_u128(*v));
                self.result()
            }

            fn i128(&mut self, v: &i128) -> Result<(), Error> {
                self.result = Some(self.serializer()?.serialize_i128(*v));
                self.result()
            }

            fn f64(&mut self, v: f64) -> Result<(), Error> {
                self.result = Some(self.serializer()?.serialize_f64(v));
                self.result()
            }

            fn bool(&mut self, v: bool) -> Result<(), Error> {
                self.result = Some(self.serializer()?.serialize_bool(v));
                self.result()
            }

            fn char(&mut self, v: char) -> Result<(), Error> {
                self.result = Some(self.serializer()?.serialize_char(v));
                self.result()
            }

            fn str(&mut self, v: &str) -> Result<(), Error> {
                self.result = Some(self.serializer()?.serialize_str(v));
                self.result()
            }

            fn none(&mut self) -> Result<(), Error> {
                self.result = Some(self.serializer()?.serialize_none());
                self.result()
            }

            #[cfg(feature = "error")]
            fn error(&mut self, v: &(dyn std::error::Error + 'static)) -> Result<(), Error> {
                self.result = Some(self.serializer()?.collect_str(v));
                self.result()
            }

            #[cfg(feature = "sval1")]
            fn sval1(&mut self, v: &dyn crate::internal::sval::v1::Value) -> Result<(), Error> {
                self.result = Some(crate::internal::sval::v1::serde(self.serializer()?, v));
                self.result()
            }

            fn serde1(&mut self, v: &dyn Serialize) -> Result<(), Error> {
                self.result = Some(erased_serde1::serialize(v, self.serializer()?));
                self.result()
            }
        }

        let mut visitor = Serde1Visitor {
            inner: Some(s),
            result: None,
        };

        self.internal_visit(&mut visitor)
            .map_err(|e| S::Error::custom(e))?;

        visitor.into_result()
    }
}

pub use erased_serde1::Serialize;

pub(in crate::internal) fn fmt(f: &mut fmt::Formatter, v: &dyn Serialize) -> Result<(), Error> {
    fmt::Debug::fmt(&serde1_fmt::to_debug(v), f)?;
    Ok(())
}

#[cfg(feature = "sval1")]
pub(in crate::internal) fn sval1(
    s: &mut sval1_lib::value::Stream,
    v: &dyn Serialize,
) -> Result<(), Error> {
    sval1_lib::serde::v1::stream(s, v).map_err(Error::from_sval1)?;
    Ok(())
}

pub(crate) fn internal_visit<'v>(
    v: &dyn Serialize,
    visitor: &mut dyn InternalVisitor<'v>,
) -> Result<(), Error> {
    struct VisitorSerializer<'a, 'v>(&'a mut dyn InternalVisitor<'v>);

    impl<'a, 'v> serde1_lib::Serializer for VisitorSerializer<'a, 'v> {
        type Ok = ();
        type Error = Unsupported;

        type SerializeSeq = Impossible<Self::Ok, Self::Error>;
        type SerializeTuple = Impossible<Self::Ok, Self::Error>;
        type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
        type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
        type SerializeMap = Impossible<Self::Ok, Self::Error>;
        type SerializeStruct = Impossible<Self::Ok, Self::Error>;
        type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

        fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
            self.0.u64(v as u64).map_err(|_| Unsupported)
        }

        fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
            self.0.u64(v as u64).map_err(|_| Unsupported)
        }

        fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
            self.0.u64(v as u64).map_err(|_| Unsupported)
        }

        fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
            self.0.u64(v).map_err(|_| Unsupported)
        }

        fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
            self.0.u128(&v).map_err(|_| Unsupported)
        }

        fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
            self.0.i64(v as i64).map_err(|_| Unsupported)
        }

        fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
            self.0.i64(v as i64).map_err(|_| Unsupported)
        }

        fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
            self.0.i64(v as i64).map_err(|_| Unsupported)
        }

        fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
            self.0.i64(v).map_err(|_| Unsupported)
        }

        fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
            self.0.i128(&v).map_err(|_| Unsupported)
        }

        fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
            self.0.f64(v as f64).map_err(|_| Unsupported)
        }

        fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
            self.0.f64(v).map_err(|_| Unsupported)
        }

        fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
            self.0.char(v).map_err(|_| Unsupported)
        }

        fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
            self.0.bool(v).map_err(|_| Unsupported)
        }

        fn serialize_some<T>(self, v: &T) -> Result<Self::Ok, Self::Error>
        where
            T: serde1_lib::Serialize + ?Sized,
        {
            v.serialize(self)
        }

        fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
            self.0.none().map_err(|_| Unsupported)
        }

        fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
            self.0.none().map_err(|_| Unsupported)
        }

        fn serialize_bytes(self, _: &[u8]) -> Result<Self::Ok, Self::Error> {
            Err(Unsupported)
        }

        fn serialize_str(self, s: &str) -> Result<Self::Ok, Self::Error> {
            self.0.str(s).map_err(|_| Unsupported)
        }

        fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
            Err(Unsupported)
        }

        fn serialize_unit_variant(
            self,
            _: &'static str,
            _: u32,
            _: &'static str,
        ) -> Result<Self::Ok, Self::Error> {
            Err(Unsupported)
        }

        fn serialize_newtype_struct<T>(
            self,
            _: &'static str,
            _: &T,
        ) -> Result<Self::Ok, Self::Error>
        where
            T: serde1_lib::Serialize + ?Sized,
        {
            Err(Unsupported)
        }

        fn serialize_newtype_variant<T>(
            self,
            _: &'static str,
            _: u32,
            _: &'static str,
            _: &T,
        ) -> Result<Self::Ok, Self::Error>
        where
            T: serde1_lib::Serialize + ?Sized,
        {
            Err(Unsupported)
        }

        fn serialize_seq(
            self,
            _: core::option::Option<usize>,
        ) -> Result<Self::SerializeSeq, Self::Error> {
            Err(Unsupported)
        }

        fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
            Err(Unsupported)
        }

        fn serialize_tuple_struct(
            self,
            _: &'static str,
            _: usize,
        ) -> Result<Self::SerializeTupleStruct, Self::Error> {
            Err(Unsupported)
        }

        fn serialize_tuple_variant(
            self,
            _: &'static str,
            _: u32,
            _: &'static str,
            _: usize,
        ) -> Result<Self::SerializeTupleVariant, Self::Error> {
            Err(Unsupported)
        }

        fn serialize_map(
            self,
            _: core::option::Option<usize>,
        ) -> Result<Self::SerializeMap, Self::Error> {
            Err(Unsupported)
        }

        fn serialize_struct(
            self,
            _: &'static str,
            _: usize,
        ) -> Result<Self::SerializeStruct, Self::Error> {
            Err(Unsupported)
        }

        fn serialize_struct_variant(
            self,
            _: &'static str,
            _: u32,
            _: &'static str,
            _: usize,
        ) -> Result<Self::SerializeStructVariant, Self::Error> {
            Err(Unsupported)
        }
    }

    erased_serde1::serialize(v, VisitorSerializer(visitor)).map_err(|_| Error::serde())
}

impl Error {
    fn serde() -> Self {
        Error::msg("`serde` serialization failed")
    }
}

#[derive(Debug)]
struct Unsupported;

impl fmt::Display for Unsupported {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid cast")
    }
}

impl serde1_lib::ser::Error for Unsupported {
    fn custom<T>(_: T) -> Self
    where
        T: fmt::Display,
    {
        Unsupported
    }
}

impl serde1_lib::ser::StdError for Unsupported {}

#[cfg(test)]
mod tests {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    use super::*;
    use crate::test::*;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn serde1_capture() {
        assert_eq!(ValueBag::capture_serde1(&42u64).to_token(), Token::U64(42));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn serde1_capture_cast() {
        assert_eq!(
            42u64,
            ValueBag::capture_serde1(&42u64)
                .to_u64()
                .expect("invalid value")
        );

        assert_eq!(
            "a string",
            ValueBag::capture_serde1(&"a string")
                .to_borrowed_str()
                .expect("invalid value")
        );

        #[cfg(feature = "std")]
        assert_eq!(
            "a string",
            ValueBag::capture_serde1(&"a string")
                .to_str()
                .expect("invalid value")
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn serde1_from_cast() {
        assert_eq!(
            42u64,
            ValueBag::from_serde1(&42u64)
                .to_u64()
                .expect("invalid value")
        );

        #[cfg(feature = "std")]
        assert_eq!(
            "a string",
            ValueBag::from_serde1(&"a string")
                .to_str()
                .expect("invalid value")
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn serde1_downcast() {
        #[derive(Debug, PartialEq, Eq)]
        struct Timestamp(usize);

        impl serde1_lib::Serialize for Timestamp {
            fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
            where
                S: serde1_lib::Serializer,
            {
                s.serialize_u64(self.0 as u64)
            }
        }

        let ts = Timestamp(42);

        assert_eq!(
            &ts,
            ValueBag::capture_serde1(&ts)
                .downcast_ref::<Timestamp>()
                .expect("invalid value")
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn serde1_structured() {
        use serde1_test::{assert_ser_tokens, Token};

        assert_ser_tokens(&ValueBag::from(42u64), &[Token::U64(42)]);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn serde1_debug() {
        struct TestSerde;

        impl serde1_lib::Serialize for TestSerde {
            fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
            where
                S: serde1_lib::Serializer,
            {
                s.serialize_u64(42)
            }
        }

        assert_eq!(
            format!("{:04?}", 42u64),
            format!("{:04?}", ValueBag::capture_serde1(&TestSerde)),
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn serde1_visit() {
        ValueBag::from_serde1(&42u64)
            .visit(TestVisit)
            .expect("failed to visit value");
        ValueBag::from_serde1(&-42i64)
            .visit(TestVisit)
            .expect("failed to visit value");
        ValueBag::from_serde1(&11f64)
            .visit(TestVisit)
            .expect("failed to visit value");
        ValueBag::from_serde1(&true)
            .visit(TestVisit)
            .expect("failed to visit value");
        ValueBag::from_serde1(&"some string")
            .visit(TestVisit)
            .expect("failed to visit value");
        ValueBag::from_serde1(&'n')
            .visit(TestVisit)
            .expect("failed to visit value");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg(feature = "sval1")]
    fn serde1_sval() {
        use sval1_lib::test::Token;

        struct TestSerde;

        impl serde1_lib::Serialize for TestSerde {
            fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
            where
                S: serde1_lib::Serializer,
            {
                s.serialize_u64(42)
            }
        }

        assert_eq!(
            vec![Token::Unsigned(42)],
            sval1_lib::test::tokens(ValueBag::capture_serde1(&TestSerde))
        );
    }

    #[cfg(feature = "std")]
    mod std_support {
        use super::*;

        use crate::std::borrow::ToOwned;

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        fn serde1_cast() {
            assert_eq!(
                "a string",
                ValueBag::capture_serde1(&"a string".to_owned())
                    .to_str()
                    .expect("invalid value")
            );
        }
    }
}
