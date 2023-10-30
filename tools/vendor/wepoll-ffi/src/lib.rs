#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
// bindgen tests are currently triggering a lint for dereferencing a null ptr when calculating
// offset of fields
// https://github.com/rust-lang/rust-bindgen/issues/1651
#[allow(unknown_lints)]
#[allow(deref_nullptr)]
#[rustfmt::skip]
pub mod bindings;
pub use bindings::*;
