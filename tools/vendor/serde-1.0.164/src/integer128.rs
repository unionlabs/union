/// Conditional compilation depending on whether Serde is built with support for
/// 128-bit integers.
///
/// Data formats that wish to support Rust compiler versions older than 1.26
/// (or targets that lack 128-bit integers) may place the i128 / u128 methods
/// of their Serializer and Deserializer behind this macro.
///
/// Data formats that require a minimum Rust compiler version of at least 1.26,
/// or do not target platforms that lack 128-bit integers, do not need to
/// bother with this macro and may assume support for 128-bit integers.
///
/// ```edition2018
/// # use serde::__private::doc::Error;
/// #
/// # struct MySerializer;
/// #
/// use serde::{serde_if_integer128, Serializer};
///
/// impl Serializer for MySerializer {
///     type Ok = ();
///     type Error = Error;
///
///     fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
///         /* ... */
/// #         unimplemented!()
///     }
///
///     /* ... */
///
///     serde_if_integer128! {
///         fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
///             /* ... */
/// #             unimplemented!()
///         }
///
///         fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
///             /* ... */
/// #             unimplemented!()
///         }
///     }
/// #
/// #     serde::__serialize_unimplemented! {
/// #         bool i8 i16 i32 u8 u16 u32 u64 f32 f64 char str bytes none some
/// #         unit unit_struct unit_variant newtype_struct newtype_variant seq
/// #         tuple tuple_struct tuple_variant map struct struct_variant
/// #     }
/// }
/// ```
///
/// When Serde is built with support for 128-bit integers, this macro expands
/// transparently into just the input tokens.
///
/// ```edition2018
/// macro_rules! serde_if_integer128 {
///     ($($tt:tt)*) => {
///         $($tt)*
///     };
/// }
/// ```
///
/// When built without support for 128-bit integers, this macro expands to
/// nothing.
///
/// ```edition2018
/// macro_rules! serde_if_integer128 {
///     ($($tt:tt)*) => {};
/// }
/// ```
#[cfg(not(no_integer128))]
#[macro_export]
macro_rules! serde_if_integer128 {
    ($($tt:tt)*) => {
        $($tt)*
    };
}

#[cfg(no_integer128)]
#[macro_export]
#[doc(hidden)]
macro_rules! serde_if_integer128 {
    ($($tt:tt)*) => {};
}
