#![recursion_limit = "128"]
// TODO: re-enable this lint when we bump msrv to 1.58
#![allow(clippy::uninlined_format_args)]

mod helpers;

mod enum_schema;
mod struct_schema;
pub use enum_schema::process_enum;
pub use struct_schema::process_struct;
