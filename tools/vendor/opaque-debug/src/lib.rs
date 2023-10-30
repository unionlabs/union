//! Macro for opaque `Debug` trait implementation.
#![no_std]

#[doc(hidden)]
pub extern crate core as __core;

/// Macro for defining opaque `Debug` implementation.
///
/// It will use the following format: "StructName { ... }". While it's
/// convinient to have it (e.g. for including into other structs), it could be
/// undesirable to leak internal state, which can happen for example through
/// uncareful logging.
#[macro_export]
macro_rules! implement {
    ($struct:ty) => {
        impl $crate::__core::fmt::Debug for $struct {
            fn fmt(&self, f: &mut $crate::__core::fmt::Formatter)
                -> Result<(), $crate::__core::fmt::Error>
            {
                write!(f, concat!(stringify!($struct), " {{ ... }}"))
            }
        }
    }
}
