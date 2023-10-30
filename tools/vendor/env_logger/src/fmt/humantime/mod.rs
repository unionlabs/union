/*
This internal module contains the timestamp implementation.

Its public API is available when the `humantime` crate is available.
*/

#[cfg_attr(feature = "humantime", path = "extern_impl.rs")]
#[cfg_attr(not(feature = "humantime"), path = "shim_impl.rs")]
mod imp;

pub(in crate::fmt) use self::imp::*;
