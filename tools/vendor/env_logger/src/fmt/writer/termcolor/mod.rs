/*
This internal module contains the style and terminal writing implementation.

Its public API is available when the `termcolor` crate is available.
The terminal printing is shimmed when the `termcolor` crate is not available.
*/

#[cfg_attr(feature = "termcolor", path = "extern_impl.rs")]
#[cfg_attr(not(feature = "termcolor"), path = "shim_impl.rs")]
mod imp;

pub(in crate::fmt) use self::imp::*;
