//! Structured values.
//!
//! This crate contains the [`ValueBag`] type, a container for an anonymous structured value.
//! `ValueBag`s can be captured in various ways and then formatted, inspected, and serialized
//! without losing their original structure.

#![cfg_attr(value_bag_capture_const_type_id, feature(const_type_id))]
#![doc(html_root_url = "https://docs.rs/value-bag/1.0.0-alpha.9")]
#![no_std]

#[cfg(any(feature = "std", test))]
#[macro_use]
#[allow(unused_imports)]
extern crate std;

#[cfg(not(any(feature = "std", test)))]
#[macro_use]
#[allow(unused_imports)]
extern crate core as std;

mod error;
pub mod fill;
mod impls;
mod internal;
pub mod visit;

#[cfg(any(test, feature = "test"))]
pub mod test;

pub use self::error::Error;

/// A dynamic structured value.
///
/// # Capturing values
///
/// There are a few ways to capture a value:
///
/// - Using the `ValueBag::capture_*` and `ValueBag::from_*` methods.
/// - Using the standard `From` trait.
/// - Using the `Fill` API.
///
/// ## Using the `ValueBag::capture_*` methods
///
/// `ValueBag` offers a few constructor methods that capture values of different kinds.
/// These methods require a `T: 'static` to support downcasting.
///
/// ```
/// use value_bag::ValueBag;
///
/// let value = ValueBag::capture_debug(&42i32);
///
/// assert_eq!(Some(42), value.to_i64());
/// ```
///
/// Capturing a value using these methods will retain type information so that
/// the contents of the bag can be serialized using an appropriate type.
///
/// For cases where the `'static` bound can't be satisfied, there's also a few
/// constructors that exclude it.
///
/// ```
/// # use std::fmt::Debug;
/// use value_bag::ValueBag;
///
/// let value = ValueBag::from_debug(&42i32);
///
/// assert_eq!(None, value.to_i64());
/// ```
///
/// These `ValueBag::from_*` methods are lossy though and `ValueBag::capture_*` should be preferred.
///
/// ## Using the standard `From` trait
///
/// Primitive types can be converted into a `ValueBag` using the standard `From` trait.
///
/// ```
/// use value_bag::ValueBag;
///
/// let value = ValueBag::from(42i32);
///
/// assert_eq!(Some(42), value.to_i64());
/// ```
///
/// ## Using the `Fill` API
///
/// The [`fill`] module provides a way to bridge APIs that may not be directly
/// compatible with other constructor methods.
///
/// The `Fill` trait is automatically implemented for closures, so can usually
/// be used in libraries that can't implement the trait themselves.
///
/// ```
/// use value_bag::{ValueBag, fill::Slot};
///
/// let value = ValueBag::from_fill(&|slot: Slot| {
///     #[derive(Debug)]
///     struct MyShortLivedValue;
///
///     slot.fill_debug(&MyShortLivedValue)
/// });
///
/// assert_eq!("MyShortLivedValue", format!("{:?}", value));
/// ```
///
/// The trait can also be implemented manually:
///
/// ```
/// # use std::fmt::Debug;
/// use value_bag::{ValueBag, Error, fill::{Slot, Fill}};
///
/// struct FillDebug;
///
/// impl Fill for FillDebug {
///     fn fill(&self, slot: Slot) -> Result<(), Error> {
///         slot.fill_debug(&42i32 as &dyn Debug)
///     }
/// }
///
/// let value = ValueBag::from_fill(&FillDebug);
///
/// assert_eq!(None, value.to_i64());
/// ```
///
/// # Inspecting values
///
/// Once you have a `ValueBag` there are also a few ways to inspect it:
///
/// - Using `std::fmt`
/// - Using `sval`
/// - Using `serde`
/// - Using the `ValueBag::visit` method.
/// - Using the `ValueBag::to_*` methods.
/// - Using the `ValueBag::downcast_ref` method.
///
/// ## Using the `ValueBag::visit` method
///
/// The [`visit`] module provides a simple visitor API that can be used to inspect
/// the structure of primitives stored in a `ValueBag`.
/// More complex datatypes can then be handled using `std::fmt`, `sval`, or `serde`.
///
/// ```
/// #[cfg(not(feature = "std"))] fn main() {}
/// #[cfg(feature = "std")]
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # fn escape(buf: &[u8]) -> &[u8] { buf }
/// # fn itoa_fmt<T>(num: T) -> Vec<u8> { vec![] }
/// # fn ryu_fmt<T>(num: T) -> Vec<u8> { vec![] }
/// # use std::io::Write;
/// use value_bag::{ValueBag, Error, visit::Visit};
///
/// // Implement some simple custom serialization
/// struct MyVisit(Vec<u8>);
/// impl<'v> Visit<'v> for MyVisit {
///     fn visit_any(&mut self, v: ValueBag) -> Result<(), Error> {
///         // Fallback to `Debug` if we didn't visit the value specially
///         write!(&mut self.0, "{:?}", v).map_err(|_| Error::msg("failed to write value"))
///     }
///
///     fn visit_u64(&mut self, v: u64) -> Result<(), Error> {
///         self.0.extend_from_slice(itoa_fmt(v).as_slice());
///         Ok(())
///     }
///
///     fn visit_i64(&mut self, v: i64) -> Result<(), Error> {
///         self.0.extend_from_slice(itoa_fmt(v).as_slice());
///         Ok(())
///     }
///
///     fn visit_f64(&mut self, v: f64) -> Result<(), Error> {
///         self.0.extend_from_slice(ryu_fmt(v).as_slice());
///         Ok(())
///     }
///
///     fn visit_str(&mut self, v: &str) -> Result<(), Error> {
///         self.0.push(b'\"');
///         self.0.extend_from_slice(escape(v.as_bytes()));
///         self.0.push(b'\"');
///         Ok(())
///     }
///
///     fn visit_bool(&mut self, v: bool) -> Result<(), Error> {
///         self.0.extend_from_slice(if v { b"true" } else { b"false" });
///         Ok(())
///     }
/// }
///
/// let value = ValueBag::from(42i64);
///
/// let mut visitor = MyVisit(vec![]);
/// value.visit(&mut visitor)?;
/// # Ok(())
/// # }
/// ```
///
/// ## Using `std::fmt`
///
/// Any `ValueBag` can be formatted using the `std::fmt` machinery as either `Debug`
/// or `Display`.
///
/// ```
/// use value_bag::ValueBag;
///
/// let value = ValueBag::from(true);
///
/// assert_eq!("true", format!("{:?}", value));
/// ```
///
/// ## Using `sval`
///
/// When the `sval1` feature is enabled, any `ValueBag` can be serialized using `sval`.
/// This makes it possible to visit any typed structure captured in the `ValueBag`,
/// including complex datatypes like maps and sequences.
///
/// `sval` doesn't need to allocate so can be used in no-std environments.
///
/// First, enable the `sval1` feature in your `Cargo.toml`:
///
/// ```toml
/// [dependencies.value-bag]
/// features = ["sval1"]
/// ```
///
/// Then stream the contents of the `ValueBag` using `sval`.
///
/// ```
/// #[cfg(not(all(feature = "std", feature = "sval1")))] fn main() {}
/// #[cfg(all(feature = "std", feature = "sval1"))]
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # extern crate sval1_json as sval_json;
/// use value_bag::ValueBag;
///
/// let value = ValueBag::from(42i64);
/// let json = sval_json::to_string(value)?;
/// # Ok(())
/// # }
/// ```
///
/// ```
/// #[cfg(not(all(feature = "std", feature = "sval1")))] fn main() {}
/// #[cfg(all(feature = "std", feature = "sval1"))]
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # extern crate sval1_lib as sval;
/// # fn escape(buf: &[u8]) -> &[u8] { buf }
/// # fn itoa_fmt<T>(num: T) -> Vec<u8> { vec![] }
/// # fn ryu_fmt<T>(num: T) -> Vec<u8> { vec![] }
/// use value_bag::ValueBag;
/// use sval::stream::{self, Stream};
///
/// // Implement some simple custom serialization
/// struct MyStream(Vec<u8>);
/// impl Stream for MyStream {
///     fn u64(&mut self, v: u64) -> stream::Result {
///         self.0.extend_from_slice(itoa_fmt(v).as_slice());
///         Ok(())
///     }
///
///     fn i64(&mut self, v: i64) -> stream::Result {
///         self.0.extend_from_slice(itoa_fmt(v).as_slice());
///         Ok(())
///     }
///
///     fn f64(&mut self, v: f64) -> stream::Result {
///         self.0.extend_from_slice(ryu_fmt(v).as_slice());
///         Ok(())
///     }
///
///     fn str(&mut self, v: &str) -> stream::Result {
///         self.0.push(b'\"');
///         self.0.extend_from_slice(escape(v.as_bytes()));
///         self.0.push(b'\"');
///         Ok(())
///     }
///
///     fn bool(&mut self, v: bool) -> stream::Result {
///         self.0.extend_from_slice(if v { b"true" } else { b"false" });
///         Ok(())
///     }
/// }
///
/// let value = ValueBag::from(42i64);
///
/// let mut stream = MyStream(vec![]);
/// sval::stream(&mut stream, &value)?;
/// # Ok(())
/// # }
/// ```
///
/// ## Using `serde`
///
/// When the `serde1` feature is enabled, any `ValueBag` can be serialized using `serde`.
/// This makes it possible to visit any typed structure captured in the `ValueBag`,
/// including complex datatypes like maps and sequences.
///
/// `serde` needs a few temporary allocations, so also brings in the `std` feature.
///
/// First, enable the `serde1` feature in your `Cargo.toml`:
///
/// ```toml
/// [dependencies.value-bag]
/// features = ["serde1"]
/// ```
///
/// Then stream the contents of the `ValueBag` using `serde`.
///
/// ```
/// #[cfg(not(all(feature = "std", feature = "serde1")))] fn main() {}
/// #[cfg(all(feature = "std", feature = "serde1"))]
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # extern crate serde1_json as serde_json;
/// use value_bag::ValueBag;
///
/// let value = ValueBag::from(42i64);
/// let json = serde_json::to_string(&value)?;
/// # Ok(())
/// # }
/// ```
///
/// Also see [`serde.rs`](https://serde.rs) for more examples of writing your own serializers.
///
/// ## Using the `ValueBag::to_*` methods
///
/// `ValueBag` provides a set of methods for attempting to pull a concrete value out.
/// These are useful for ad-hoc analysis but aren't intended for exhaustively serializing
/// the contents of a `ValueBag`.
///
/// ```
/// use value_bag::ValueBag;
///
/// let value = ValueBag::capture_display(&42u64);
///
/// assert_eq!(Some(42u64), value.to_u64());
/// ```
///
/// ## Using the `ValueBag::downcast_ref` method
///
/// When a `ValueBag` is created using one of the `capture_*` constructors, it can be downcast
/// back to its original value.
/// This can also be useful for ad-hoc analysis where there's a common possible non-primitive
/// type that could be captured.
///
/// ```
/// # #[derive(Debug)] struct SystemTime;
/// # fn now() -> SystemTime { SystemTime }
/// use value_bag::ValueBag;
///
/// let timestamp = now();
/// let value = ValueBag::capture_debug(&timestamp);
///
/// assert!(value.downcast_ref::<SystemTime>().is_some());
/// ```
#[derive(Clone)]
pub struct ValueBag<'v> {
    inner: internal::Internal<'v>,
}

impl<'v> ValueBag<'v> {
    /// Get a `ValueBag` from a reference to a `ValueBag`.
    #[inline]
    pub fn by_ref<'u>(&'u self) -> ValueBag<'u> {
        ValueBag { inner: self.inner }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::std::mem;

    #[test]
    fn value_bag_size() {
        let size = mem::size_of::<ValueBag<'_>>();
        let limit = mem::size_of::<u64>() * 3;

        if size > limit {
            panic!(
                "`ValueBag` size ({} bytes) is too large (expected up to {} bytes)\n`(`&dyn` + `TypeId`): {} bytes",
                size,
                limit,
                mem::size_of::<(&dyn internal::fmt::Debug, crate::std::any::TypeId)>(),
            );
        }
    }
}
