//! Value inspection.
//!
//! The [`Visit`] trait provides a simple visitor API that can be used to inspect
//! the structure of primitives stored in a [`ValueBag`](../struct.ValueBag.html).
//! More complex datatypes can then be handled using `std::fmt`, `sval`, or `serde`.
//!
//! ```
//! #[cfg(not(feature = "std"))] fn main() {}
//! #[cfg(feature = "std")]
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # fn escape(buf: &[u8]) -> &[u8] { buf }
//! # fn itoa_fmt<T>(num: T) -> Vec<u8> { vec![] }
//! # fn ryu_fmt<T>(num: T) -> Vec<u8> { vec![] }
//! # use std::io::Write;
//! use value_bag::{ValueBag, Error, visit::Visit};
//!
//! // Implement some simple custom serialization
//! struct MyVisit(Vec<u8>);
//! impl<'v> Visit<'v> for MyVisit {
//!     fn visit_any(&mut self, v: ValueBag) -> Result<(), Error> {
//!         // Fallback to `Debug` if we didn't visit the value specially
//!         write!(&mut self.0, "{:?}", v).map_err(|_| Error::msg("failed to write value"))
//!     }
//!
//!     fn visit_u64(&mut self, v: u64) -> Result<(), Error> {
//!         self.0.extend_from_slice(itoa_fmt(v).as_slice());
//!         Ok(())
//!     }
//!
//!     fn visit_i64(&mut self, v: i64) -> Result<(), Error> {
//!         self.0.extend_from_slice(itoa_fmt(v).as_slice());
//!         Ok(())
//!     }
//!
//!     fn visit_f64(&mut self, v: f64) -> Result<(), Error> {
//!         self.0.extend_from_slice(ryu_fmt(v).as_slice());
//!         Ok(())
//!     }
//!
//!     fn visit_str(&mut self, v: &str) -> Result<(), Error> {
//!         self.0.push(b'\"');
//!         self.0.extend_from_slice(escape(v.as_bytes()));
//!         self.0.push(b'\"');
//!         Ok(())
//!     }
//!
//!     fn visit_bool(&mut self, v: bool) -> Result<(), Error> {
//!         self.0.extend_from_slice(if v { b"true" } else { b"false" });
//!         Ok(())
//!     }
//! }
//!
//! let value = ValueBag::from(42i64);
//!
//! let mut visitor = MyVisit(vec![]);
//! value.visit(&mut visitor)?;
//! # Ok(())
//! # }
//! ```

use crate::{
    internal::{self, InternalVisitor},
    Error, ValueBag,
};

/// A visitor for a `ValueBag`.
pub trait Visit<'v> {
    /// Visit a `ValueBag`.
    ///
    /// This is the only required method on `Visit` and acts as a fallback for any
    /// more specific methods that aren't overridden.
    /// The `ValueBag` may be formatted using its `fmt::Debug` or `fmt::Display` implementation,
    /// or serialized using its `sval::Value` or `serde::Serialize` implementation.
    fn visit_any(&mut self, value: ValueBag) -> Result<(), Error>;

    /// Visit an unsigned integer.
    #[inline]
    #[cfg(not(test))]
    fn visit_u64(&mut self, value: u64) -> Result<(), Error> {
        self.visit_any(value.into())
    }
    #[cfg(test)]
    fn visit_u64(&mut self, value: u64) -> Result<(), Error>;

    /// Visit a signed integer.
    #[inline]
    #[cfg(not(test))]
    fn visit_i64(&mut self, value: i64) -> Result<(), Error> {
        self.visit_any(value.into())
    }
    #[cfg(test)]
    fn visit_i64(&mut self, value: i64) -> Result<(), Error>;

    /// Visit a big unsigned integer.
    #[inline]
    #[cfg(not(test))]
    fn visit_u128(&mut self, value: u128) -> Result<(), Error> {
        self.visit_any((&value).into())
    }
    #[cfg(test)]
    fn visit_u128(&mut self, value: u128) -> Result<(), Error>;

    /// Visit a big signed integer.
    #[inline]
    #[cfg(not(test))]
    fn visit_i128(&mut self, value: i128) -> Result<(), Error> {
        self.visit_any((&value).into())
    }
    #[cfg(test)]
    fn visit_i128(&mut self, value: i128) -> Result<(), Error>;

    /// Visit a floating point.
    #[inline]
    #[cfg(not(test))]
    fn visit_f64(&mut self, value: f64) -> Result<(), Error> {
        self.visit_any(value.into())
    }
    #[cfg(test)]
    fn visit_f64(&mut self, value: f64) -> Result<(), Error>;

    /// Visit a boolean.
    #[inline]
    #[cfg(not(test))]
    fn visit_bool(&mut self, value: bool) -> Result<(), Error> {
        self.visit_any(value.into())
    }
    #[cfg(test)]
    fn visit_bool(&mut self, value: bool) -> Result<(), Error>;

    /// Visit a string.
    #[inline]
    #[cfg(not(test))]
    fn visit_str(&mut self, value: &str) -> Result<(), Error> {
        self.visit_any(value.into())
    }
    #[cfg(test)]
    fn visit_str(&mut self, value: &str) -> Result<(), Error>;

    /// Visit a string.
    #[inline]
    #[cfg(not(test))]
    fn visit_borrowed_str(&mut self, value: &'v str) -> Result<(), Error> {
        self.visit_str(value)
    }
    #[cfg(test)]
    fn visit_borrowed_str(&mut self, value: &'v str) -> Result<(), Error>;

    /// Visit a Unicode character.
    #[inline]
    #[cfg(not(test))]
    fn visit_char(&mut self, value: char) -> Result<(), Error> {
        let mut b = [0; 4];
        self.visit_str(&*value.encode_utf8(&mut b))
    }
    #[cfg(test)]
    fn visit_char(&mut self, value: char) -> Result<(), Error>;

    /// Visit an error.
    #[inline]
    #[cfg(not(test))]
    #[cfg(feature = "error")]
    fn visit_error(&mut self, err: &(dyn crate::std::error::Error + 'static)) -> Result<(), Error> {
        self.visit_any(ValueBag::from_dyn_error(err))
    }
    #[cfg(test)]
    #[cfg(feature = "error")]
    fn visit_error(&mut self, err: &(dyn crate::std::error::Error + 'static)) -> Result<(), Error>;

    /// Visit an error.
    #[inline]
    #[cfg(not(test))]
    #[cfg(feature = "error")]
    fn visit_borrowed_error(
        &mut self,
        err: &'v (dyn crate::std::error::Error + 'static),
    ) -> Result<(), Error> {
        self.visit_any(ValueBag::from_dyn_error(err))
    }
    #[cfg(test)]
    #[cfg(feature = "error")]
    fn visit_borrowed_error(
        &mut self,
        err: &'v (dyn crate::std::error::Error + 'static),
    ) -> Result<(), Error>;
}

impl<'a, 'v, T: ?Sized> Visit<'v> for &'a mut T
where
    T: Visit<'v>,
{
    #[inline]
    fn visit_any(&mut self, value: ValueBag) -> Result<(), Error> {
        (**self).visit_any(value)
    }

    #[inline]
    fn visit_u64(&mut self, value: u64) -> Result<(), Error> {
        (**self).visit_u64(value)
    }

    #[inline]
    fn visit_i64(&mut self, value: i64) -> Result<(), Error> {
        (**self).visit_i64(value)
    }

    #[inline]
    fn visit_u128(&mut self, value: u128) -> Result<(), Error> {
        (**self).visit_u128(value)
    }

    #[inline]
    fn visit_i128(&mut self, value: i128) -> Result<(), Error> {
        (**self).visit_i128(value)
    }

    #[inline]
    fn visit_f64(&mut self, value: f64) -> Result<(), Error> {
        (**self).visit_f64(value)
    }

    #[inline]
    fn visit_bool(&mut self, value: bool) -> Result<(), Error> {
        (**self).visit_bool(value)
    }

    #[inline]
    fn visit_str(&mut self, value: &str) -> Result<(), Error> {
        (**self).visit_str(value)
    }

    #[inline]
    fn visit_borrowed_str(&mut self, value: &'v str) -> Result<(), Error> {
        (**self).visit_borrowed_str(value)
    }

    #[inline]
    fn visit_char(&mut self, value: char) -> Result<(), Error> {
        (**self).visit_char(value)
    }

    #[inline]
    #[cfg(feature = "error")]
    fn visit_error(&mut self, err: &(dyn crate::std::error::Error + 'static)) -> Result<(), Error> {
        (**self).visit_error(err)
    }

    #[inline]
    #[cfg(feature = "error")]
    fn visit_borrowed_error(
        &mut self,
        err: &'v (dyn crate::std::error::Error + 'static),
    ) -> Result<(), Error> {
        (**self).visit_borrowed_error(err)
    }
}

impl<'v> ValueBag<'v> {
    /// Visit this value using a simple visitor.
    ///
    /// The visitor isn't strictly required to inspect the contents of a value bag.
    /// It's useful for simple cases where a full framework like `serde` or `sval`
    /// isn't necessary.
    pub fn visit(&self, visitor: impl Visit<'v>) -> Result<(), Error> {
        struct Visitor<V>(V);

        impl<'v, V> InternalVisitor<'v> for Visitor<V>
        where
            V: Visit<'v>,
        {
            fn debug(&mut self, v: &dyn internal::fmt::Debug) -> Result<(), Error> {
                self.0.visit_any(ValueBag::from_dyn_debug(v))
            }

            fn display(&mut self, v: &dyn internal::fmt::Display) -> Result<(), Error> {
                self.0.visit_any(ValueBag::from_dyn_display(v))
            }

            fn u64(&mut self, v: u64) -> Result<(), Error> {
                self.0.visit_u64(v)
            }

            fn i64(&mut self, v: i64) -> Result<(), Error> {
                self.0.visit_i64(v)
            }

            fn u128(&mut self, v: &u128) -> Result<(), Error> {
                self.0.visit_u128(*v)
            }

            fn i128(&mut self, v: &i128) -> Result<(), Error> {
                self.0.visit_i128(*v)
            }

            fn f64(&mut self, v: f64) -> Result<(), Error> {
                self.0.visit_f64(v)
            }

            fn bool(&mut self, v: bool) -> Result<(), Error> {
                self.0.visit_bool(v)
            }

            fn char(&mut self, v: char) -> Result<(), Error> {
                self.0.visit_char(v)
            }

            fn str(&mut self, v: &str) -> Result<(), Error> {
                self.0.visit_str(v)
            }

            fn borrowed_str(&mut self, v: &'v str) -> Result<(), Error> {
                self.0.visit_borrowed_str(v)
            }

            fn none(&mut self) -> Result<(), Error> {
                self.0.visit_any(ValueBag::from(()))
            }

            #[cfg(feature = "error")]
            fn error(&mut self, v: &(dyn internal::error::Error + 'static)) -> Result<(), Error> {
                self.0.visit_error(v)
            }

            #[cfg(feature = "error")]
            fn borrowed_error(
                &mut self,
                v: &'v (dyn internal::error::Error + 'static),
            ) -> Result<(), Error> {
                self.0.visit_borrowed_error(v)
            }

            #[cfg(feature = "sval1")]
            fn sval1(&mut self, v: &dyn internal::sval::v1::Value) -> Result<(), Error> {
                internal::sval::v1::internal_visit(v, self)
            }

            #[cfg(feature = "serde1")]
            fn serde1(&mut self, v: &dyn internal::serde::v1::Serialize) -> Result<(), Error> {
                internal::serde::v1::internal_visit(v, self)
            }
        }

        self.internal_visit(&mut Visitor(visitor))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::*;

    #[test]
    fn visit_structured() {
        ValueBag::from(42u64)
            .visit(TestVisit)
            .expect("failed to visit value");
        ValueBag::from(-42i64)
            .visit(TestVisit)
            .expect("failed to visit value");
        ValueBag::from(&42u128)
            .visit(TestVisit)
            .expect("failed to visit value");
        ValueBag::from(&-42i128)
            .visit(TestVisit)
            .expect("failed to visit value");
        ValueBag::from(11f64)
            .visit(TestVisit)
            .expect("failed to visit value");
        ValueBag::from(true)
            .visit(TestVisit)
            .expect("failed to visit value");
        ValueBag::from("some string")
            .visit(TestVisit)
            .expect("failed to visit value");
        ValueBag::from('n')
            .visit(TestVisit)
            .expect("failed to visit value");
    }
}
